import { invoke } from '@tauri-apps/api/core'

export interface LockFile {
  /** Path to the lockfile. */
  path: string
  /** League client process ID. */
  pid: number
  /** HTTP port. */
  port: number
  /** HTTP auth password. */
  token: string
  /** HTTP basic auth header value. */
  authHeader: string
}

export interface ClientSummoner {
  accountId: number
  displayName: string
  internalName: string
  nameChangeFlag: boolean
  percentCompleteForNextLevel: number
  profileIconId: number
  puuid: string
  rerollPoints: RerollPoints
  summonerId: number
  summonerLevel: number
  unnamed: boolean
  xpSinceLastLevel: number
  xpUntilNextLevel: number
}

export interface RerollPoints {
  current_points: number
  maxRolls: number
  numberOfRolls: number
  pointsCostToRoll: number
  pointsToReroll: number
}

export interface RunePage {
  current: boolean
  id: number
  isActive: boolean
  isDeletable: boolean
  isEditable: boolean
  isValid: boolean
  lastModified: number
  name: string
  order: number
  primaryStyleId: number
  selectedPerkIds: number[]
  subStyleId: number
}

export interface NewRunePage {
  name: string
  primaryStyleId: number
  selectedPerkIds: number[]
  subStyleId: number
}

export const getCurrentSummoner = async () => {
  return await invoke<ClientSummoner>('plugin:lcu|get_current_summoner')
}

export const getCurrentRunePage = async () => {
  return await invoke<RunePage>('plugin:lcu|get_current_rune_page')
}

export const updateRunePage = async (pageId: number, runePage: NewRunePage) => {
  return await invoke<null>('plugin:lcu|update_rune_page', { pageId, runePage })
}
