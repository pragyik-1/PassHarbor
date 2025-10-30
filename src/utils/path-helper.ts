import path from 'node:path'
import fs from 'node:fs/promises'
import type { Stats } from 'node:fs'
import os from 'node:os'
import { randomUUID } from 'node:crypto'
import { glob } from 'glob'

class Path {
  protected _path: string

  protected constructor(p: string | Path) {
    this._path = path.normalize(typeof p === 'string' ? p : p.toString())
  }

  static at(p: string | Path): Path {
    return new Path(p)
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

  async isFile(): Promise<boolean> {
    try {
      return (await fs.stat(this._path)).isFile()
    } catch {
      return false
    }
  }

  async isDirectory(): Promise<boolean> {
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

  /* Takes any type of path including Files and Dirs and coerces it as a different one. */
  as<T extends Path>(type: { at: (p: string) => T }): T {
    return type.at(this.toString())
  }
}

class File extends Path {
  protected constructor(p: string | Path) {
    super(p)
  }

  static at(p: string | Path): File {
    return new File(p)
  }

  async exists(): Promise<boolean> {
    return this.isFile()
  }

  async read(encoding: BufferEncoding = 'utf-8'): Promise<string> {
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

  async create(recursive = true): Promise<this> {
    await fs.mkdir(path.dirname(this.toString()), { recursive })
    await fs.writeFile(this.toString(), '')
    return this
  }

  async ensure(): Promise<this> {
    if (!(await this.exists())) {
      await this.create()
    }
    return this
  }
}

class Directory extends Path {
  protected constructor(p: string | Path) {
    super(p)
  }

  static at(p: string | Path): Directory {
    return new Directory(p)
  }

  async exists(): Promise<boolean> {
    return this.isDirectory()
  }

  async listFiles(): Promise<File[]> {
    const entries = await fs.readdir(this.toString(), { withFileTypes: true })
    const files = entries.filter((entry) => entry.isFile())
    return files.map((entry) => File.at(path.join(this.toString(), entry.name)))
  }

  // Lists directories in the current directory only.
  async listDirs(): Promise<Directory[]> {
    const entries = await fs.readdir(this.toString(), { withFileTypes: true })
    const dirs = entries.filter((entry) => entry.isDirectory())
    return dirs.map((entry) => new Directory(path.join(this.toString(), entry.name)))
  }

  // Lists files and directories in the current directory only.
  async list(): Promise<(File | Directory)[]> {
    const entries = await fs.readdir(this.toString(), { withFileTypes: true })
    const filesAndDirs: (File | Directory)[] = []
    for (const entry of entries) {
      const fullPath = path.join(this.toString(), entry.name)
      if (entry.isFile()) {
        filesAndDirs.push(File.at(fullPath))
      } else if (entry.isDirectory()) {
        filesAndDirs.push(new Directory(fullPath))
      }
    }
    return filesAndDirs
  }

  private async _listAllDeep(dirPath: string): Promise<{ files: File[]; dirs: Directory[] }> {
    const allFiles: File[] = []
    const allDirs: Directory[] = []
    const entries = await fs.readdir(dirPath, { withFileTypes: true })

    const promises = entries.map(async (entry) => {
      const fullPath = path.join(dirPath, entry.name)
      if (entry.isFile()) {
        allFiles.push(File.at(fullPath))
        return Promise.resolve()
      } else if (entry.isDirectory()) {
        allDirs.push(new Directory(fullPath))
        return this._listAllDeep(fullPath)
      }
      // If none of the above, return a resolved promise to be safe
      return Promise.resolve()
    })

    const nestedResults = await Promise.all(promises)

    for (const result of nestedResults) {
      // Check if the result is valid before trying to merge
      if (result && 'files' in result && 'dirs' in result) {
        allFiles.push(...result.files)
        allDirs.push(...result.dirs)
      }
    }

    return { files: allFiles, dirs: allDirs }
  }
  async listFilesDeep(): Promise<File[]> {
    return (await this._listAllDeep(this.toString())).files
  }

  async listDirsDeep(): Promise<Directory[]> {
    return (await this._listAllDeep(this.toString())).dirs
  }

  async listDeep(): Promise<(File | Directory)[]> {
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

  async create(recursive = true): Promise<this> {
    await fs.mkdir(this.toString(), { recursive })
    return this
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

  async walk(callback: (item: File | Directory) => Promise<void> | void): Promise<void> {
    const items = await this.list()
    for (const item of items) {
      await callback(item)
      if (item instanceof Directory) {
        await item.walk(callback)
      }
    }
  }

  async ensure(): Promise<this> {
    if (!(await this.exists())) {
      await this.create()
    }
    return this
  }
}

class Json extends File {
  protected constructor(p: string | Path) {
    super(p)
  }

  static at(p: string | Path): Json {
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

  async create(recursive = true): Promise<this> {
    await fs.mkdir(path.dirname(this.toString()), { recursive })
    await fs.writeFile(this.toString(), '{}')
    return this
  }
}

class TempDir extends Directory {
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

export { Path, File, Directory, Json, TempDir }