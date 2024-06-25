export interface APIOutput {
  success: boolean;
  error: String;
  games_included: boolean;
  reviews_included: boolean;
  games: Game[];
}
export enum LoadingState {
  WaitingForLink,
  VerifyingLink,
  ScrapingAndGuessing,
  SortingOutput,
  Failed,
  Finished,
}
export interface Game {
  name: String;
  steam_appid: number;
  score: number;
  is_free: boolean;
  short_description: String;
  developers: String[] | null;
  header_image: String;
  release_date: ReleaseDate;
  platforms: Platforms;
  price_overview: PriceOverview | null;
  content_descriptors: ContentDescriptors;
}
export interface PriceOverview {
  final_formatted: String;
}
export interface ContentDescriptors {
  ids: number[];
  notes: String | null;
}
export interface ReleaseDate {
  coming_soon: boolean;
  date: String;
}
export interface Platforms {
  windows: boolean;
  mac: boolean;
  linux: boolean;
}
