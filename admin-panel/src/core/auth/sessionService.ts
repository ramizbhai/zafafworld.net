export interface AuthUser {
  id: string;
  role: string;
  scopes?: string[];
  [key: string]: any;
}

/**
 * Validates whether the provided user object has an administrative role.
 */
export function isAdmin(user?: AuthUser | null): boolean {
  return !!user && user.role?.toLowerCase() === 'admin';
}

/**
 * Checks if a user has a specific scope.
 */
export function hasScope(user: AuthUser | null | undefined, scope: string): boolean {
  if (!user || !user.scopes || !Array.isArray(user.scopes)) {
    return false;
  }
  return user.scopes.includes(scope);
}

/**
 * Validates the profile API response format.
 */
export function isValidProfileResponse(profileData: any): boolean {
  return profileData && profileData.status === 'success' && !!profileData.user;
}

// TODO: wire into +layout.server.ts in next phase
