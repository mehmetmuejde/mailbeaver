import { vi } from "vitest";
import type {
  Folder,
  FolderView,
  FolderViews,
  LogEntry,
  LogLevel,
  MessageDetail,
  Project,
  ProjectInput,
  ServerSettings,
  Settings,
} from "@/shared/api";

const FAKE_SERVER_INFO = {
  version: "1.2.3",
  smtpAddress: "127.0.0.1:1026",
  httpAddress: "127.0.0.1:8026",
  database: "/data/mailbeaver.db",
  maxMessages: 5000,
  maxMessageBytes: 25 * 1024 * 1024,
  fromEnvironment: ["httpPort"],
  host: "desktop",
};

let nextId = 1;

/** A complete message as the detail endpoint returns it. */
export function mailFixture(overrides: Partial<MessageDetail> = {}): MessageDetail {
  const id = overrides.id ?? nextId++;
  return {
    id,
    receivedAt: 0,
    from: "Alice <alice@example.com>",
    to: ["bob@example.com"],
    subject: `Message ${id}`,
    size: 2048,
    hasAttachments: false,
    isRead: false,
    folder: "inbox",
    project: null,
    preview: "Hello Bob",
    envelopeFrom: "alice@example.com",
    envelopeTo: ["bob@example.com"],
    authUser: null,
    headers: [{ name: "Subject", value: `Message ${id}` }],
    text: "Hello Bob",
    html: null,
    attachments: [],
    ...overrides,
  };
}

const json = (body: unknown, code = 200) => Response.json(body, { status: code });
const status = (code: number) => new Response(null, { status: code });

function summary(message: MessageDetail) {
  const { envelopeFrom, envelopeTo, authUser, headers, text, html, attachments, ...rest } = message;
  return rest;
}

function counts(messages: readonly MessageDetail[]) {
  return {
    total: messages.length,
    unread: messages.filter((message) => !message.isRead).length,
  };
}

/** In-memory fake of the HTTP API, following the server's rules. */
const LOG_SEVERITY: Record<LogLevel, number> = { error: 0, warn: 1, info: 2 };

export function createFakeMailApi(
  initial: readonly MessageDetail[],
  initialProjects: readonly Project[] = [],
  initialLogs: readonly LogEntry[] = [],
) {
  let logs = [...initialLogs];
  let capture = { paused: false };
  let serverSettings: ServerSettings = {
    smtpPort: null,
    httpPort: null,
    maxMessages: null,
    maxMessageMb: null,
  };
  let messages = initial.map((message) => ({ ...message }));
  let projects = initialProjects.map((project) => ({ ...project }));
  const defaultSettings: Settings = {
    trashRetentionDays: null,
    trayGrouping: "all",
    theme: "system",
    sidebarCollapsed: false,
    notifications: "all",
    notificationSound: false,
    unreadBadge: true,
    updateCheck: false,
  };
  let settings = defaultSettings;
  const defaultView: FolderView = { grouping: "all", project: null, period: null, pageSize: 50 };
  let views: FolderViews = { inbox: defaultView, archive: defaultView, trash: defaultView };
  let nextProjectId = Math.max(0, ...projects.map((project) => project.id)) + 1;

  const update = (id: number, change: Partial<MessageDetail>) => {
    messages = messages.map((message) => (message.id === id ? { ...message, ...change } : message));
  };
  const inFolder = (folder: Folder) => messages.filter((message) => message.folder === folder);

  const listMessages = (params: URLSearchParams) => {
    const folder = params.get("folder");
    const project = params.get("project");
    const q = params.get("q")?.toLowerCase();
    const since = Number(params.get("since") ?? 0);
    const limit = Number(params.get("limit") ?? 50);
    const offset = Number(params.get("offset") ?? 0);
    const matching = messages.filter(
      (message) =>
        (folder === null || message.folder === folder) &&
        (project === null ||
          (project === "none"
            ? message.project === null
            : message.project?.id === Number(project))) &&
        (q === undefined ||
          `${message.subject} ${message.from} ${message.text ?? ""}`.toLowerCase().includes(q)) &&
        message.receivedAt >= since,
    );
    return json({
      ...counts(matching),
      messages: matching.slice(offset, offset + limit).map(summary),
    });
  };

  const projectOverview = () => {
    const outsideTrash = messages.filter((message) => message.folder !== "trash");
    return json({
      projects: projects.map((project) => ({
        ...project,
        ...counts(outsideTrash.filter((message) => message.project?.id === project.id)),
      })),
      unassigned: counts(outsideTrash.filter((message) => message.project === null)),
    });
  };

  const saveProject = (id: number | null, input: ProjectInput) => {
    const taken = projects.find(
      (project) => project.id !== id && project.name.toLowerCase() === input.name.toLowerCase(),
    );
    if (taken !== undefined) {
      return json({ error: `a project named "${input.name}" already exists` }, 409);
    }
    const project = { id: id ?? nextProjectId++, ...input };
    projects =
      id === null ? [...projects, project] : projects.map((p) => (p.id === id ? project : p));
    return json(project, id === null ? 201 : 200);
  };

  const messageRoute = (method: string, id: number, action: string | undefined, body: unknown) => {
    const message = messages.find((candidate) => candidate.id === id);
    if (message === undefined) {
      return status(404);
    }
    if (method === "GET" && action === undefined) {
      return json(message);
    }
    if (method === "GET" && action === "raw") {
      return new Response(`Subject: ${message.subject}\r\n\r\n${message.text ?? ""}`);
    }
    if (method === "POST" && action === "read") {
      update(id, { isRead: true });
      return status(204);
    }
    if (method === "POST" && action === "unread") {
      update(id, { isRead: false });
      return status(204);
    }
    if (method === "POST" && action === "move") {
      update(id, { folder: (body as { folder: Folder }).folder });
      return status(204);
    }
    if (method === "DELETE" && action === undefined) {
      if (message.folder !== "trash") {
        return status(409);
      }
      messages = messages.filter((candidate) => candidate.id !== id);
      return status(204);
    }
    return status(404);
  };

  const fetchMock = vi.fn(async (input: RequestInfo | URL, init?: RequestInit) => {
    const url = new URL(String(input), "http://localhost");
    const method = init?.method ?? "GET";
    const body: unknown = typeof init?.body === "string" ? JSON.parse(init.body) : undefined;
    const route = `${method} ${url.pathname}`;
    const messagePath = url.pathname.match(/^\/api\/messages\/(\d+)(?:\/(\w+))?$/);

    if (messagePath !== null) {
      return messageRoute(method, Number(messagePath[1]), messagePath[2], body);
    }
    const viewPath = url.pathname.match(/^\/api\/folder-views\/(inbox|archive|trash)$/);
    if (viewPath !== null && method === "PUT") {
      const folder = viewPath[1] as Folder;
      views = { ...views, [folder]: body as FolderView };
      return json(views[folder]);
    }
    const projectPath = url.pathname.match(/^\/api\/projects\/(\d+)$/);
    if (projectPath !== null) {
      const id = Number(projectPath[1]);
      if (!projects.some((project) => project.id === id)) {
        return status(404);
      }
      if (method === "PUT") {
        return saveProject(id, body as ProjectInput);
      }
      projects = projects.filter((project) => project.id !== id);
      messages = messages.map((message) =>
        message.project?.id === id ? { ...message, project: null } : message,
      );
      return status(204);
    }
    switch (route) {
      case "GET /api/messages":
        return listMessages(url.searchParams);
      case "DELETE /api/messages":
        messages = [];
        return status(204);
      case "DELETE /api/data":
        messages = [];
        projects = [];
        settings = defaultSettings;
        views = { inbox: defaultView, archive: defaultView, trash: defaultView };
        return status(204);
      case "DELETE /api/trash":
        messages = messages.filter((message) => message.folder !== "trash");
        return status(204);
      case "POST /api/messages/mark-read": {
        const folder = (body as { folder: Folder | null }).folder;
        messages = messages.map((message) =>
          folder === null || message.folder === folder ? { ...message, isRead: true } : message,
        );
        return json({ count: 0 });
      }
      case "GET /api/stats":
        return json({
          startedAt: 0,
          smtpAccepted: messages.length,
          smtpAcceptedBytes: 4096,
          smtpRejected: 1,
          messagesDeleted: 2,
          databaseBytes: 98_304,
        });
      case "GET /api/folders":
        return json({
          inbox: counts(inFolder("inbox")),
          archive: counts(inFolder("archive")),
          trash: counts(inFolder("trash")),
        });
      case "GET /api/projects":
        return projectOverview();
      case "GET /api/projects/rule-suggestions": {
        const kind = url.searchParams.get("kind");
        const prefix = (url.searchParams.get("q") ?? "").toLowerCase();
        const values = messages
          .filter((message) => message.folder !== "trash")
          .map((message) => {
            const address = (message.from.match(/<([^>]+)>/)?.[1] ?? message.from).toLowerCase();
            return kind === "sender_domain" ? (address.split("@")[1] ?? "") : address;
          })
          .filter((value) => kind !== "auth_user" && value !== "" && value.startsWith(prefix));
        const counted = new Map<string, number>();
        for (const value of values) {
          counted.set(value, (counted.get(value) ?? 0) + 1);
        }
        return json([...counted].map(([value, count]) => ({ value, count })));
      }
      case "POST /api/projects":
        return saveProject(null, body as ProjectInput);
      case "GET /api/folder-views":
        return json(views);
      case "GET /api/settings":
        return json(settings);
      case "PUT /api/settings":
        settings = body as Settings;
        return json(settings);
      case "GET /api/info":
        return json(FAKE_SERVER_INFO);
      case "GET /api/server-settings":
        return json(serverSettings);
      case "PUT /api/server-settings": {
        const next = body as ServerSettings;
        if (next.smtpPort !== null && next.smtpPort === next.httpPort) {
          return json({ error: "SMTP and HTTP need different ports" }, 422);
        }
        serverSettings = next;
        return json(serverSettings);
      }
      case "GET /api/update":
        return json({
          currentVersion: "1.2.3",
          latestVersion: settings.updateCheck ? "1.3.0" : null,
          url: settings.updateCheck
            ? "https://github.com/mehmetmuejde/mailbeaver/releases/tag/v1.3.0"
            : null,
          updateAvailable: settings.updateCheck,
          checkedAt: settings.updateCheck ? 0 : null,
        });
      case "GET /api/capture":
        return json(capture);
      case "PUT /api/capture":
        capture = body as typeof capture;
        return json(capture);
      case "GET /api/logs": {
        const minLevel = (url.searchParams.get("minLevel") ?? "info") as LogLevel;
        const source = url.searchParams.get("source");
        return json(
          logs
            .filter((entry) => LOG_SEVERITY[entry.level] <= LOG_SEVERITY[minLevel])
            .filter((entry) => source === null || entry.source === source)
            .toSorted((a, b) => b.id - a.id),
        );
      }
      case "POST /api/logs": {
        const entry = body as Pick<LogEntry, "level" | "target" | "message">;
        logs = [...logs, { ...entry, id: logs.length + 1, createdAt: 0, source: "frontend" }];
        return status(204);
      }
      case "DELETE /api/logs":
        logs = [];
        return status(204);
      default:
        return status(404);
    }
  });

  return {
    fetchMock,
    messages: () => messages,
    settings: () => settings,
    projects: () => projects,
    logs: () => logs,
    capture: () => capture,
    serverSettings: () => serverSettings,
    views: () => views,
    calls: (method: string, path: string) =>
      fetchMock.mock.calls.filter(
        ([input, init]) =>
          new URL(String(input), "http://localhost").pathname === path &&
          (init?.method ?? "GET") === method,
      ),
  };
}
