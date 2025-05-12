export interface TrackSearch {
  success: boolean;
  data: TrackSearchData;
}

export interface TrackSearchData {
  total: number;
  start: number;
  results: TrackResult[];
}

export interface TrackList {
  success: boolean;
  data: TrackResult[];
}

export interface TrackResult {
  id: string;
  name: string;
  kind: string;
  year?: string | null;
  releaseDate?: string | null;
  duration?: number | null;
  label?: string | null;
  explicitContent: boolean;
  playCount?: number | null;
  language: string;
  hasLyrics: boolean;
  lyricsId?: string | null;
  url: string;
  copyright?: string | null;
  album: Album;
  artists: Artists;
  image: ImageItem[];
  downloadUrl: DownloadUrlItem[];
}

export interface Album {
  id?: string | null;
  name?: string | null;
  url?: string | null;
}

export interface Artists {
  primary: Artist[];
  featured: Artist[];
  all: Artist[];
}

export interface Artist {
  id: string;
  name: string;
  role: string;
  kind: string;
  image: ImageItem[];
  url: string;
}

export interface ImageItem {
  quality: string;
  url: string;
}

export interface DownloadUrlItem {
  quality: string;
  url: string;
}
