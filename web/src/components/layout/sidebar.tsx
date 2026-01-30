"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useTheme } from "next-themes";
import {
  LayoutDashboard,
  CheckSquare,
  Bot,
  Puzzle,
  Settings,
  PanelLeftClose,
  PanelLeft,
  Sun,
  Moon,
  Monitor,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { usePorterStore } from "@/lib/store";
import { cn } from "@/lib/utils";

const navigation = [
  { name: "Dashboard", href: "/", icon: LayoutDashboard },
  { name: "Tasks", href: "/tasks", icon: CheckSquare },
  { name: "Agents", href: "/agents", icon: Bot },
  { name: "Skills", href: "/skills", icon: Puzzle },
  { name: "Settings", href: "/settings", icon: Settings },
];

const themeOrder = ["system", "light", "dark"] as const;
const themeIcon = {
  system: Monitor,
  light: Sun,
  dark: Moon,
};
const themeLabel = {
  system: "System",
  light: "Light",
  dark: "Dark",
};

export function Sidebar() {
  const pathname = usePathname();
  const { sidebarOpen, toggleSidebar } = usePorterStore();
  const { theme, setTheme } = useTheme();

  const cycleTheme = () => {
    const current = themeOrder.indexOf(
      (theme as (typeof themeOrder)[number]) ?? "system"
    );
    const next = themeOrder[(current + 1) % themeOrder.length];
    setTheme(next);
  };

  const ThemeIcon = themeIcon[(theme as keyof typeof themeIcon) ?? "system"] ?? Monitor;

  return (
    <aside
      className={cn(
        "flex flex-col border-r border-border bg-card transition-all duration-200",
        sidebarOpen ? "w-60" : "w-16"
      )}
    >
      <div className="flex h-14 items-center justify-between px-4">
        {sidebarOpen && (
          <h1 className="text-lg font-semibold tracking-tight">Porter</h1>
        )}
        <Button
          variant="ghost"
          size="icon"
          onClick={toggleSidebar}
          className="h-8 w-8"
        >
          {sidebarOpen ? (
            <PanelLeftClose className="h-4 w-4" />
          ) : (
            <PanelLeft className="h-4 w-4" />
          )}
        </Button>
      </div>

      <Separator />

      <nav className="flex-1 space-y-1 p-2">
        {navigation.map((item) => {
          const isActive =
            item.href === "/"
              ? pathname === "/"
              : pathname.startsWith(item.href);

          return (
            <Link
              key={item.name}
              href={item.href}
              className={cn(
                "flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors",
                isActive
                  ? "bg-accent text-accent-foreground"
                  : "text-muted-foreground hover:bg-accent hover:text-accent-foreground"
              )}
            >
              <item.icon className="h-4 w-4 shrink-0" />
              {sidebarOpen && <span>{item.name}</span>}
            </Link>
          );
        })}
      </nav>

      <Separator />

      <div className="p-2">
        <button
          onClick={cycleTheme}
          className="flex w-full items-center gap-3 rounded-md px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground"
        >
          <ThemeIcon className="h-4 w-4 shrink-0" />
          {sidebarOpen && <span>{themeLabel[(theme as keyof typeof themeLabel) ?? "system"] ?? "System"}</span>}
        </button>
      </div>
    </aside>
  );
}
