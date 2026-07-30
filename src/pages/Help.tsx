import { Tabs, TabsList, TabsTrigger, TabsContent } from "@/components/ui/tabs";
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Markdown } from "@/components/Markdown";
import { PageHeader } from "@/components/PageHeader";
import { CopyButton } from "@/components/CopyButton";

import gettingStartedMd from "@/help/getting-started.md?raw";
import mcpServerMd from "@/help/mcp-server.md?raw";
import llmProvidersMd from "@/help/llm-providers.md?raw";
import troubleshootingMd from "@/help/troubleshooting.md?raw";
import cliToolMd from "@/help/cli-tool.md?raw";

const MCP_CONFIG = `{
  "mcpServers": {
    "nootle": {
      "command": "/Applications/Nootle.app/Contents/MacOS/nootle",
      "args": ["--mcp"]
    }
  }
}`;

function McpQuickStart() {
  return (
    <Card>
      <CardHeader>
        <CardTitle>Quick Start — MCP Config</CardTitle>
        <CardDescription>
          Add this to your Claude Code config to connect Nootle:
        </CardDescription>
      </CardHeader>
      <CardContent>
        <div className="relative">
          <pre className="overflow-x-auto rounded-lg bg-muted p-4 font-mono text-xs">
            {MCP_CONFIG}
          </pre>
          <CopyButton
            variant="button"
            text={MCP_CONFIG}
            className="absolute top-2 right-2"
          />
        </div>
      </CardContent>
    </Card>
  );
}

const tabs = [
  { value: "getting-started", label: "Getting Started", content: gettingStartedMd },
  { value: "mcp-server", label: "MCP Server", content: mcpServerMd, quickStart: true },
  { value: "cli-tool", label: "CLI Tool", content: cliToolMd },
  { value: "llm-providers", label: "LLM Providers", content: llmProvidersMd },
  { value: "troubleshooting", label: "Troubleshooting", content: troubleshootingMd },
] as const;

export function HelpPage() {
  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <PageHeader
        title="Help"
        description="Learn how to use Nootle and get the most out of your meetings"
      />

      <Tabs defaultValue="getting-started" className="flex flex-1 flex-col overflow-hidden">
        <div className="shrink-0 border-b px-6 py-4">
          <TabsList className="h-10">
            {tabs.map((tab) => (
              <TabsTrigger key={tab.value} value={tab.value}>
                {tab.label}
              </TabsTrigger>
            ))}
          </TabsList>
        </div>

        {tabs.map((tab) => (
          <TabsContent key={tab.value} value={tab.value} className="mt-0 flex-1 overflow-auto">
            <div className="flex max-w-3xl flex-col gap-6 p-6">
              {"quickStart" in tab && tab.quickStart && <McpQuickStart />}
              <Card>
                <CardContent>
                  <Markdown content={tab.content} />
                </CardContent>
              </Card>
            </div>
          </TabsContent>
        ))}
      </Tabs>
    </div>
  );
}
