import { create } from "zustand";
import { persist } from "zustand/middleware";
import { api, setExplicitToken } from "@/lib/api";
import { UserProfile } from "@/types";

interface AuthState {
  user: UserProfile | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;

  fetchSession: () => Promise<void>;
  loginAsAnalyst: () => Promise<void>;
  logout: () => void;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: {
        id: "0e8224c9-1c1b-4fd1-a2d5-e092967453a5",
        name: "Lead Risk Analyst",
        email: "analyst@capitalcore.bank",
        role: "Risk & Compliance Officer",
        department: "Global Fraud Operations",
        permissions: [
          "events:read",
          "events:write",
          "decisions:read",
          "rules:read",
          "audit:read",
        ],
      },
      token: null,
      isAuthenticated: true,
      isLoading: false,

      fetchSession: async () => {
        try {
          set({ isLoading: true });
          const data = await api.get<{ access_token: string; user: UserProfile }>("/auth/session");
          setExplicitToken(data.access_token);
          set({
            token: data.access_token,
            user: data.user,
            isAuthenticated: true,
            isLoading: false,
          });
        } catch {
          set({ isLoading: false });
        }
      },

      loginAsAnalyst: async () => {
        await get().fetchSession();
      },

      logout: () => {
        setExplicitToken(null);
        get().fetchSession().catch(() => {});
      },
    }),
    {
      name: "capitalcore-auth",
      partialize: (s) => ({ token: s.token, user: s.user, isAuthenticated: s.isAuthenticated }),
    }
  )
);
