import { Path, File, Dir } from '../utils/path-helper'
import { app } from 'electron'

export const RootDir = new Path(`${app.getPath('userData')}/logsafe`)
export const VaultsDir = new Dir(RootDir.join('vaults'))
export const MasterPasswordPath = new File(RootDir.join('mpwd.txt'))
