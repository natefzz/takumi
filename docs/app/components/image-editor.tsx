import { Editor } from "@monaco-editor/react";
import { fromJsx } from "@takumi-rs/helpers/jsx";
import DocsTemplateV1 from "@takumi-rs/template/docs-template-v1";
import type { AnyNode } from "@takumi-rs/wasm";
import * as React from "react";
import { useEffect, useRef, useState } from "react";
import { transform } from "sucrase";
import defaultTemplate from "~/playground/default?raw";
import TakumiWorker from "~/playground/worker?worker";
import {
  ResizableHandle,
  ResizablePanel,
  ResizablePanelGroup,
} from "./ui/resizable";

function transformCode(code: string) {
  return transform(code, {
    transforms: ["jsx", "typescript", "imports"],
    production: true,
  }).code;
}

function require(module: string) {
  if (module === "@takumi-rs/template/docs-template-v1") return DocsTemplateV1;
}

export default function ImageEditor() {
  const [code, setCode] = useState(defaultTemplate);
  const Component = React.useMemo(() => {
    const exports = {};

    try {
      new Function("exports", "require", "React", transformCode(code))(
        exports,
        require,
        React,
      );

      if (!("default" in exports) || typeof exports.default !== "function")
        throw new Error("Default export should be a React component.");

      return exports.default as React.JSXElementConstructor<unknown>;
    } catch (e) {
      console.error(e);
      return () => <></>;
    }
  }, [code]);

  const [node, setNode] = useState<AnyNode>();
  const [rendered, setRendered] = useState<string>();
  const [isReady, setIsReady] = useState(false);
  const workerRef = useRef<Worker | undefined>(undefined);

  useEffect(() => {
    const worker = new TakumiWorker();

    worker.onmessage = (event: MessageEvent) => {
      if (event.data.type === "ready") {
        setIsReady(true);
      } else if (event.data.type === "render_complete") {
        setRendered(event.data.dataUrl);
      } else if (event.data.type === "render_error") {
        console.error("Worker render error:", event.data.error);
      }
    };

    workerRef.current = worker;

    return () => {
      worker.terminate();
      workerRef.current = undefined;
      setIsReady(false);
    };
  }, []);

  useEffect(() => {
    fromJsx(<Component />).then((node) => {
      setNode(node);

      if (isReady) {
        workerRef.current?.postMessage({
          type: "render",
          node,
        });
      }
    });
  }, [Component, isReady]);

  return (
    <div className="h-[calc(100dvh-3.5rem)]">
      <ResizablePanelGroup direction="horizontal">
        <ResizablePanel defaultSize={50}>
          <Editor
            onMount={(_, monaco) => {
              monaco.languages.typescript.typescriptDefaults.setDiagnosticsOptions(
                {
                  noSemanticValidation: true,
                  noSyntaxValidation: true,
                  noSuggestionDiagnostics: true,
                },
              );
            }}
            width="100%"
            height="100%"
            language="typescript"
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
              fontSize: 16,
            }}
            loading="Launching editor..."
            defaultValue={code}
            onChange={(code) => setCode(code ?? "")}
          />
        </ResizablePanel>
        <ResizableHandle withHandle />
        <ResizablePanel defaultSize={50}>
          <ResizablePanelGroup direction="vertical">
            <ResizablePanel
              defaultSize={50}
              className="flex justify-center items-center"
            >
              {rendered && <img src={rendered} alt="Takumi rendered result" />}
            </ResizablePanel>
            <ResizableHandle withHandle />
            <ResizablePanel defaultSize={50}>
              <div className="h-full overflow-y-auto p-4">
                <p className="text-lg py-2 font-medium">What Takumi Sees</p>
                <pre>{JSON.stringify(node, null, 2)}</pre>
              </div>
            </ResizablePanel>
          </ResizablePanelGroup>
        </ResizablePanel>
      </ResizablePanelGroup>
    </div>
  );
}
