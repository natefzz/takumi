import { Editor } from "@monaco-editor/react";
import { shikiToMonaco } from "@shikijs/monaco";
import { useMemo, useRef } from "react";
import { createHighlighterCore } from "shiki/core";
import { createOnigurumaEngine } from "shiki/engine-oniguruma.mjs";
import takumiTypings from "../../../node_modules/@takumi-rs/wasm/pkg/takumi_wasm.d.ts?raw";
import reactTypings from "../../../node_modules/@types/react/index.d.ts?raw";
import reactJsxRuntimeTypings from "../../../node_modules/@types/react/jsx-runtime.d.ts?raw";
import cssTypings from "../../../node_modules/csstype/index.d.ts?raw";
import playgroundOptionsTypings from "../../playground/options.ts?raw";

function createHighlighter() {
  return createHighlighterCore({
    themes: [import("shiki/themes/github-dark-default.mjs")],
    langs: [import("shiki/langs/tsx.mjs")],
    engine: createOnigurumaEngine(import("shiki/wasm")),
    langAlias: {
      typescript: "tsx",
    },
  });
}

type GlobalThis = typeof globalThis & {
  shikiInstance: ReturnType<typeof createHighlighter>;
};

(globalThis as GlobalThis).shikiInstance ??= createHighlighter();

const highlighter = await (globalThis as GlobalThis).shikiInstance;

const tailwindTypings = `
declare namespace React {
  interface HTMLAttributes<T> {
    tw?: string;
  }
}
`;

export function ComponentEditor({
  code,
  setCode,
}: {
  code: string;
  setCode: (code: string) => void;
}) {
  const codeRef = useRef(code);

  const memorized = useMemo(
    () => (
      <Editor
        beforeMount={(monaco) => {
          monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
            target: monaco.languages.typescript.ScriptTarget.Latest,
            allowNonTsExtensions: true,
            moduleResolution:
              monaco.languages.typescript.ModuleResolutionKind.NodeJs,
            module: monaco.languages.typescript.ModuleKind.ESNext,
            reactNamespace: "React",
            esModuleInterop: true,
            jsx: monaco.languages.typescript.JsxEmit.ReactJSX,
            typeRoots: ["node_modules/@types"],
          });

          monaco.languages.typescript.typescriptDefaults.setExtraLibs([
            {
              content: reactTypings,
              filePath: "file:///node_modules/react/index.d.ts",
            },
            {
              content: reactJsxRuntimeTypings,
              filePath: "file:///node_modules/react/jsx-runtime.d.ts",
            },
            {
              content: cssTypings,
              filePath: "file:///node_modules/csstype/index.d.ts",
            },
            {
              content: takumiTypings,
              filePath: "file:///node_modules/@takumi-rs/wasm/index.d.ts",
            },
            {
              content: playgroundOptionsTypings,
              filePath: "file:///options.d.ts",
            },
            {
              content: tailwindTypings,
              filePath: "file:///tw.d.ts",
            },
          ]);

          shikiToMonaco(highlighter, monaco);
        }}
        width="100%"
        height="100%"
        language="typescript"
        theme="github-dark-default"
        path="main.tsx"
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
          fontSize: 16,
          padding: {
            top: 8,
            bottom: 8,
          },
          scrollBeyondLastLine: false,
        }}
        loading="Launching editor..."
        defaultValue={codeRef.current}
        onChange={(value) => setCode(value ?? "")}
      />
    ),
    [setCode],
  );

  return memorized;
}
