export interface ApiSuccess<T> {
  data: T;
}

export interface ApiList<T> {
  data: T[];
  cursor: string | null;
  has_more: boolean;
}

export interface ApiError {
  error: {
    code: string;
    message: string;
  };
}

export type ApiResponse<T> = ApiSuccess<T> | ApiError;
export type ApiListResponse<T> = ApiList<T> | ApiError;
