import type { Connection } from "../../lib/schema";

const LABEL = { working: "Working", attention: "Needs attention", broken: "Not working" } as const;

function host(url: string): string {
  try {
    return new URL(url).host;
  } catch {
    return url;
  }
}

function lastSync(connection: Connection): string {
  if (!connection.lastSync) {
    return "Never synced";
  }

  const at = new Date(connection.lastSync.at);
  return `Last sync ${at.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" })}`;
}

export function ConnectionRow({ connection }: { connection: Connection }) {
  const { health, problems } = connection;

  return (
    <li className="connection">
      <div className="connection-head">
        <span className={`dot dot-${health}`} role="img" aria-label={LABEL[health]} />
        <span className="connection-name">{connection.name}</span>
        <span className="connection-host">{host(connection.url)}</span>
      </div>
      <div className="connection-detail" title={connection.configDir}>
        {connection.configDir}
      </div>
      <div className="connection-detail">{lastSync(connection)}</div>
      {problems.map((problem) => (
        <div key={problem} className={`problem problem-${health}`}>
          {problem}
        </div>
      ))}
    </li>
  );
}
