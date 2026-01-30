"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function SkillsPage() {
  const { data: skills, isLoading } = useQuery({
    queryKey: ["skills"],
    queryFn: api.listSkills,
  });

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Skills</h1>
        <p className="text-muted-foreground">
          Installed skills and capabilities
        </p>
      </div>

      {isLoading ? (
        <div className="grid gap-4 md:grid-cols-2">
          {[...Array(2)].map((_, i) => (
            <div
              key={i}
              className="h-32 rounded-lg border bg-card animate-pulse"
            />
          ))}
        </div>
      ) : skills && skills.length > 0 ? (
        <div className="grid gap-4 md:grid-cols-2">
          {skills.map((skill) => (
            <Card key={skill.id}>
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-base">{skill.name}</CardTitle>
                  <Badge variant={skill.enabled ? "default" : "secondary"}>
                    {skill.enabled ? "Active" : "Disabled"}
                  </Badge>
                </div>
              </CardHeader>
              <CardContent>
                <div className="flex flex-wrap gap-1">
                  {skill.capabilities.map((cap) => (
                    <Badge key={cap} variant="outline" className="text-xs">
                      {cap}
                    </Badge>
                  ))}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <div className="rounded-lg border border-dashed p-8 text-center text-muted-foreground">
          No skills configured. Enable skills in your config file.
        </div>
      )}
    </div>
  );
}
