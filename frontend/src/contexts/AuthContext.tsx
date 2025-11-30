import React, {
  createContext,
  useContext,
  useEffect,
  useState,
  useCallback,
  ReactNode,
} from "react";
import { API_ENDPOINTS } from "../config/api";

interface User {
  username: string;
  role: string;
}

interface AuthContextType {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  login: (
    username: string,
    password: string,
  ) => Promise<{ success: boolean; error?: string }>;
  logout: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

interface AuthProviderProps {
  children: ReactNode;
}

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const [user, setUser] = useState<User | null>(null);
  const [token, setToken] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  const validateToken = useCallback(async (authToken: string) => {
    try {
      const response = await fetch(`${API_ENDPOINTS.API}/blockchain/status`, {
        headers: {
          Authorization: `Bearer ${authToken}`,
          "Content-Type": "application/json",
        },
      });

      if (response.ok) {
        // Token is valid, extract user info from JWT payload
        const payload = parseJWT(authToken);
        setUser({
          username: payload.sub,
          role: payload.role,
        });
        setToken(authToken);
      } else {
        // Token is invalid, remove it
        localStorage.removeItem("authToken");
      }
    } catch (error) {
      console.error("Token validation failed:", error);
      localStorage.removeItem("authToken");
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Check for existing token on mount
  useEffect(() => {
    const storedToken = localStorage.getItem("authToken");
    if (storedToken) {
      // Validate token by making a request to a protected endpoint
      validateToken(storedToken);
    } else {
      setIsLoading(false);
    }
  }, [validateToken]);

  const parseJWT = (token: string) => {
    try {
      // Validate JWT structure first
      const parts = token.split(".");
      if (parts.length !== 3) {
        throw new Error("Invalid JWT structure");
      }

      const base64Url = parts[1];
      if (!base64Url) {
        throw new Error("JWT payload missing");
      }

      const base64 = base64Url.replace(/-/g, "+").replace(/_/g, "/");

      // Handle padding for base64 decoding
      const paddedBase64 = base64.padEnd(
        base64.length + ((4 - (base64.length % 4)) % 4),
        "=",
      );

      const decodedPayload = atob(paddedBase64);

      // More robust URI component decoding
      let jsonPayload;
      try {
        jsonPayload = decodeURIComponent(
          decodedPayload
            .split("")
            .map((c) => {
              const charCode = c.charCodeAt(0);
              // Only encode characters that need encoding
              if (charCode < 128 && (charCode < 32 || charCode > 126)) {
                return "%" + ("00" + charCode.toString(16)).slice(-2);
              }
              return c;
            })
            .join(""),
        );
      } catch {
        // Fallback to using the raw decoded string if URI decoding fails
        jsonPayload = decodedPayload;
      }

      return JSON.parse(jsonPayload);
    } catch (error) {
      console.error("Failed to parse JWT:", error);
      return { sub: "", role: "" };
    }
  };

  const login = async (
    username: string,
    password: string,
  ): Promise<{ success: boolean; error?: string }> => {
    try {
      const response = await fetch(`${API_ENDPOINTS.AUTH}/login`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          username: username.trim(),
          password: password,
        }),
      });

      if (response.ok) {
        const authData = await response.json();
        const authToken = authData.token;

        // Store token
        localStorage.setItem("authToken", authToken);

        // Use a microtask to ensure state updates are batched properly
        Promise.resolve().then(() => {
          setToken(authToken);
          setUser({
            username: username,
            role: authData.user_role,
          });
        });

        return { success: true };
      } else {
        const errorData = await response.json().catch(() => ({}));
        return {
          success: false,
          error: errorData.message || "Invalid credentials. Please try again.",
        };
      }
    } catch (error) {
      console.error("Login error:", error);
      return {
        success: false,
        error: "Network error. Please check your connection and try again.",
      };
    }
  };

  const logout = () => {
    localStorage.removeItem("authToken");
    setUser(null);
    setToken(null);
  };

  const value: AuthContextType = {
    user,
    token,
    isAuthenticated: !!token,
    isLoading,
    login,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

// Export the hook separately to avoid fast refresh warning
export const useAuth = () => {
  const context = useContext(AuthContext);
  if (context === undefined) {
    throw new Error("useAuth must be used within an AuthProvider");
  }
  return context;
};

export default AuthContext;
