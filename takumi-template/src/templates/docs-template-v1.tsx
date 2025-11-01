import type { ReactNode } from "react";

// Source: https://github.com/fuma-nama/fumadocs/blob/bb47513e4350fd898b2a53f735538a247b4b660c/packages/ui/src/og.tsx
export default function DocsTemplateV1({
  title,
  description,
  icon,
  primaryColor,
  primaryTextColor,
  site,
}: {
  title: ReactNode;
  description: ReactNode;
  icon: ReactNode;
  primaryColor: string;
  primaryTextColor: string;
  site: ReactNode;
}) {
  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        width: "100%",
        height: "100%",
        color: "white",
        padding: "4rem",
        backgroundColor: "#0c0c0c",
        backgroundImage: `linear-gradient(to top right, ${primaryColor}, transparent), noise-v1(opacity(0.2))`,
      }}
    >
      <div
        style={{
          display: "flex",
          flexDirection: "row",
          alignItems: "center",
          gap: "16px",
          marginBottom: "12px",
          color: primaryTextColor,
        }}
      >
        {icon}
        <span
          style={{
            fontSize: 56,
            fontWeight: 600,
          }}
        >
          {site}
        </span>
      </div>
      <p
        style={{
          fontWeight: 800,
          fontSize: 84,
          textOverflow: "ellipsis",
          lineClamp: 2,
          marginTop: "0.5em",
          marginBottom: "0.5em",
        }}
      >
        {title}
      </p>
      <span
        style={{
          fontSize: 48,
          color: "rgba(240,240,240,0.8)",
          fontWeight: 500,
          lineClamp: 2,
          textOverflow: "ellipsis",
        }}
      >
        {description}
      </span>
    </div>
  );
}
