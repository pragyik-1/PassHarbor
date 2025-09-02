import path from 'node:path'
import fs from 'node:fs/promises'
import type { Stats } from 'node:fs'
import os from 'node:os'
import { randomUUID } from 'node:crypto'
import { glob } from 'glob'

class Path {
  protected _path: string

  constructor(p: string | Path) {
    this._path = path.normalize(typeof p === 'string' ? p : p.toString())
  }

  valueOf(): string {
    return this._path
  }

  toString(): string {
    return this._path
  }

  join(...paths: string[]): Path {
    return new Path(path.join(this._path, ...paths))
  }

  basename(): string {
    return path.basename(this._path)
  }

  name(): string {
    return path.basename(this._path, path.extname(this._path))
  }

  ext(): string {
    return path.extname(this._path)
  }

  isAbsolute(): boolean {
    return path.isAbsolute(this._path)
  }

  resolve(): Path {
    return new Path(path.resolve(this._path))
  }

  relativeTo(to: string): Path {
    return new Path(path.relative(to, this._path))
  }

  parent(): Path {
    return new Path(path.dirname(this._path))
  }

  withExt(newExt: string): Path {
    const basename = path.basename(this._path, path.extname(this._path))
    return new Path(path.join(path.dirname(this._path), `${basename}${newExt}`))
  }

  withName(newName: string): Path {
    return new Path(path.join(path.dirname(this._path), newName))
  }

  async exists(): Promise<boolean> {
    try {
      await fs.access(this._path)
      return true
    } catch {
      return false
    }
  }

  async isFile() {
    try {
      return (await fs.stat(this._path)).isFile()
    } catch {
      return false
    }
  }

  async isDirectory() {
    try {
      return (await fs.stat(this._path)).isDirectory()
    } catch {
      return false
    }
  }

  async stats(): Promise<Stats> {
    return fs.stat(this._path)
  }

  async lstats(): Promise<Stats> {
    return fs.lstat(this._path)
  }

  as<T extends Path>(type: { __from: (p: string) => T }): T {
    return type.__from(this.toString())
  }
}

class File extends Path {
  constructor(p: string | Path) {
    super(p)
  }

  static __from(p: string | Path): File {
    return new File(p)
  }

  async exists() {
    try {
      return (await fs.stat(this.toString())).isFile()
    } catch {
      return false
    }
  }

  async read(encoding: BufferEncoding = 'utf-8'): Promise<any> {
    return fs.readFile(this.toString(), encoding)
  }

  async write(data: string | Buffer, encoding: BufferEncoding = 'utf-8'): Promise<void> {
    await fs.writeFile(this.toString(), data, encoding)
  }

  async size(): Promise<number> {
    const stats = await fs.stat(this.toString())
    return stats.size
  }

  async remove(): Promise<void> {
    await fs.rm(this.toString())
  }

  async renameTo(newName: string): Promise<void> {
    await fs.rename(this.toString(), newName)
  }

  async copyTo(target: string): Promise<void> {
    await fs.copyFile(this.toString(), target)
  }

  async moveTo(target: string): Promise<void> {
    await fs.rename(this.toString(), target)
  }

  async create(recursive = true): Promise<void> {
    await fs.mkdir(path.dirname(this.toString()), { recursive })
    await fs.writeFile(this.toString(), '')
  }
}

class Dir extends Path {
  constructor(p: string | Path) {
    super(p)
  }

  static __from(p: string | Path): Dir {
    return new Dir(p)
  }

  async exists() {
    try {
      return (await fs.stat(this.toString())).isDirectory()
    } catch {
      return false
    }
  }

  async listFiles(): Promise<File[]> {
    const entries = await fs.readdir(this.toString(), { withFileTypes: true })
    const files = entries.filter((entry) => entry.isFile())
    return files.map((entry) => new File(path.join(this.toString(), entry.name)))
  }

  // Lists directories in the current directory only.
  async listDirs(): Promise<Dir[]> {
    const entries = await fs.readdir(this.toString(), { withFileTypes: true })
    const dirs = entries.filter((entry) => entry.isDirectory())
    return dirs.map((entry) => new Dir(path.join(this.toString(), entry.name)))
  }

  // Lists files and directories in the current directory only.
  async list(): Promise<(File | Dir)[]> {
    const entries = await fs.readdir(this.toString(), { withFileTypes: true })
    const filesAndDirs: (File | Dir)[] = []
    for (const entry of entries) {
      const fullPath = path.join(this.toString(), entry.name)
      if (entry.isFile()) {
        filesAndDirs.push(new File(fullPath))
      } else if (entry.isDirectory()) {
        filesAndDirs.push(new Dir(fullPath))
      }
    }
    return filesAndDirs
  }

  private async _listAllDeep(dirPath: string): Promise<{ files: File[]; dirs: Dir[] }> {
    const allFiles: File[] = []
    const allDirs: Dir[] = []
    const entries = await fs.readdir(dirPath, { withFileTypes: true })

    //
    const promises = entries.map(async (entry) => {
      const fullPath = path.join(dirPath, entry.name)
      if (entry.isFile()) {
        allFiles.push(new File(fullPath))
      } else if (entry.isDirectory()) {
        allDirs.push(new Dir(fullPath))
        // Recursively call for subdirectories and return the promise
        return this._listAllDeep(fullPath)
      }
    })

    // Wait for all promises to resolve
    const nestedResults = await Promise.all(promises)

    // Merge the results from the nested directories
    for (const result of nestedResults) {
      if (result) {
        allFiles.push(...result.files)
        allDirs.push(...result.dirs)
      }
    }

    return { files: allFiles, dirs: allDirs }
  }
  async listFilesDeep(): Promise<File[]> {
    return (await this._listAllDeep(this.toString())).files
  }

  async listDirsDeep(): Promise<Dir[]> {
    return (await this._listAllDeep(this.toString())).dirs
  }

  async listDeep(): Promise<(File | Dir)[]> {
    const { files, dirs } = await this._listAllDeep(this.toString())
    return [...files, ...dirs]
  }

  async remove(): Promise<void> {
    await fs.rm(this.toString(), { recursive: true })
  }

  async renameTo(newName: string): Promise<void> {
    await fs.rename(this.toString(), newName)
  }

  async copyTo(target: string): Promise<void> {
    await fs.cp(this.toString(), target, { recursive: true })
  }

  async create(recursive = true): Promise<void> {
    await fs.mkdir(this.toString(), { recursive })
  }

  async moveTo(target: string): Promise<void> {
    await fs.rename(this.toString(), target)
  }

  async empty(): Promise<void> {
    const items = await fs.readdir(this.toString())
    await Promise.all(
      items.map((item) => fs.rm(path.join(this.toString(), item), { recursive: true, force: true }))
    )
  }

  async glob(pattern: string): Promise<Path[]> {
    const results = await glob(pattern, { cwd: this.toString() })
    return results.map((p: string) => new Path(path.join(this.toString(), p)))
  }

  // In Dir class
  async walk(callback: (item: File | Dir) => Promise<void> | void): Promise<void> {
    const items = await this.list()
    for (const item of items) {
      await callback(item)
      if (item instanceof Dir) {
        await item.walk(callback)
      }
    }
  }
}

class Json extends File {
  constructor(p: string | Path) {
    super(p)
    if (this.ext() !== '.json') {
      throw new Error(`Not a valid JSON file: ${this.toString()}`)
    }
  }

  static __from(p: string | Path): Json {
    return new Json(p)
  }

  async read<T>(): Promise<T> {
    return JSON.parse(await super.read('utf-8'))
  }

  async write(data: unknown, encoding: BufferEncoding = 'utf-8'): Promise<void> {
    if (typeof data === 'object' && !Buffer.isBuffer(data)) {
      return super.write(JSON.stringify(data, null, 2), encoding)
    }
    return super.write(JSON.stringify(data), encoding)
  }

  async create(recursive = true): Promise<void> {
    await fs.mkdir(path.dirname(this.toString()), { recursive })
    await fs.writeFile(this.toString(), '{}')
  }
}

class TempDir extends Dir {
  constructor() {
    const tempPath = path.join(os.tmpdir(), `temp-${randomUUID()}`)
    super(tempPath)
  }
  async with<T>(fn: (dir: TempDir) => Promise<T>): Promise<T> {
    await this.create()
    try {
      return await fn(this)
    } finally {
      await this.remove()
    }
  }
}

export { Path, File, Dir, Json, TempDir }
