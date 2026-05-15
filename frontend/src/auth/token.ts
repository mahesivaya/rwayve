let inMemoryToken: string | null = null;

export const getAuthToken = () => inMemoryToken;

export const setAuthToken = (token: string) => {
  inMemoryToken = token;
};

export const clearAuthToken = () => {
  inMemoryToken = null;
  localStorage.removeItem("token");
};
