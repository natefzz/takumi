import DocsTemplateV1 from "@takumi-rs/template/docs-template-v1";

export default function Image() {
  return (
    <DocsTemplateV1
      title="Takumi Playground"
      description="You can try out and make quick prototypes here."
      site="Takumi"
      icon={
        <div
          style={{
            width: "4rem",
            height: "4rem",
            borderRadius: "50%",
            justifyContent: "center",
            backgroundColor: "red",
            fontSize: 48,
            fontWeight: 600,
            color: "white",
            textAlign: "center",
          }}
        />
      }
      primaryColor="red"
      primaryTextColor="red"
    />
  );
}
