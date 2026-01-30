"use client";

import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function SettingsPage() {
  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Settings</h1>
        <p className="text-muted-foreground">Configure your Porter instance</p>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Instance Configuration</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            Settings are managed via TOML config files. Edit your config file
            and restart the server to apply changes.
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
