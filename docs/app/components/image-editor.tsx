import { Editor } from "@monaco-editor/react";
import { useState } from "react";

export default function ImageEditor() {
  const [code, setCode] = useState<string>();

  return (
    <div className="h-[calc(100dvh-3.5rem)] relative">
      <Editor
        width="100%"
        height="100%"
        language="tsx"
        theme="vs-dark"
        options={{
          wordWrap: "on",
          tabSize: 2,
          minimap: {
            enabled: false,
          },
          stickyScroll: {
            enabled: false,
          },
          scrollbar: {
            useShadows: false,
          },
        }}
        defaultValue={code}
        onChange={setCode}
      />
    </div>
  );
}
