import { Channel } from '@tauri-apps/api/core'

export const api = $state<{
  channel?: Channel<KruggMessage>
  champs?: ChampionShort[]
  champ?: Champion
  overview?: { overview: OverviewData; role: Role }
  matchups?: { matchups: MatchupData; role: Role }
}>({})

export const newChannel = () => {
  api.channel = new Channel<KruggMessage>()
  api.channel.onmessage = (evt) => {
    console.log('Channel event: type:', evt.type, ', data:', evt.data)

    switch (evt.type) {
      case 'champions':
        api.champs = Object.entries(evt.data)
          .map(([, champ]) => champ)
          .toSorted(({ name: a }, { name: b }) => a.localeCompare(b))
        break
      case 'champion':
        api.champ = evt.data
        break
      case 'overview':
        api.overview = evt.data
        break
      case 'matchups':
        api.matchups = evt.data
    }
  }

  return api.channel
}

export type KruggMessage =
  | {
      type: 'champions'
      data: Record<string, ChampionShort>
    }
  | {
      type: 'champion'
      data: Champion
    }
  | {
      type: 'overview'
      data: {
        overview: OverviewData
        role: Role
      }
    }
  | {
      type: 'matchups'
      data: {
        matchups: MatchupData
        role: Role
      }
    }

export enum Role {
  Jungle = 1,
  Support,
  ADCarry,
  Top,
  Mid,
  /** Only used for ARAM. */
  None,
  Automatic,
  /** Only used for Nexus Blitz. */
  Lane,
}

export interface ChampionShort {
  version: string
  id: string
  key: string
  name: string
  title: string
  blurb: string
  info: Info
  image: Image
  tags: Tag[]
  partype: string
  stats: Record<string, number>
}

export interface Info {
  attack: number
  defense: number
  magic: number
  difficulty: number
}

export interface Image {
  full: string
  sprite: string
  group: string
  x: number
  y: number
  w: number
  h: number
}

export enum Tag {
  Assassin,
  Fighter,
  Mage,
  Marksman,
  Support,
  Tank,
}

export interface OverviewData {
  runes: Runes
  summonerSpells: SummonerSpells
  startingItems: Items
  coreItems: Items
  abilities: Abilities
  item4Options: LateItem[]
  item5Options: LateItem[]
  item6Options: LateItem[]
  wins: number
  matches: number
  lowSampleSize: boolean
  shards: Shards
}

export interface Runes {
  matches: number
  wins: number
  primaryStyleId: number
  secondaryStyleId: number
  runeIds: number[]
}

export interface SummonerSpells {
  matches: number
  wins: number
  spellIds: number[]
}

export interface Items {
  matches: number
  wins: number
  itemIds: number[]
}

export interface Abilities {
  matches: number
  wins: number
  abilityOrder: string[]
  abilityMaxOrder: string
}

export interface LateItem {
  matches: number
  wins: number
  id: number
}

export interface Shards {
  matches: number
  wins: number
  shardIds: number[]
}

export interface MatchupData {
  bestMatchups: Matchup[]
  worstMatchups: Matchup[]
  totalMatches: number
}

export interface Matchup {
  championId: number
  wins: number
  matches: number
  winrate: number
}

export interface Champion {
  id: string
  key: string
  name: string
  title: string
  image: Image
  skins: Skin[]
  lore: string
  blurb: string
  allytips: string[]
  enemytips: string[]
  tags: Tag[]
  partype: string
  info: Info
  stats: Record<string, number>
  spells: Spell[]
  passive: Passive
}

export interface Skin {
  id: string
  num: number
  name: string
  chromas: boolean
}

export interface Spell {
  id: string
  name: string
  description: string
  tooltip: string
  leveltip: LevelTip | null
  maxrank: number
  cooldown: number[]
  cooldownBurn: string
  cost: number[]
  costBurn: string
  effect: (number[] | null)[]
  effectBurn: (string | null)[]
  costType: string
  maxammo: string
  range: number[]
  rangeBurn: string
  image: Image
  resource: string | null
}

export interface LevelTip {
  label: string[]
  effect: string[]
}

export interface Passive {
  name: string
  description: string
  image: Image
}
