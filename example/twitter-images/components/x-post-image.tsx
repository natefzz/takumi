import { file } from "bun";

export const name = "x-post-image";

export const width = 1200;
export const height = 630;

export const fonts = [];

export const persistentImages = [
  {
    src: "takumi.svg",
    data: await file("../../assets/images/takumi.svg").arrayBuffer(),
  },
  {
    src: "fuma.jpg",
    data: await fetch(
      "https://pbs.twimg.com/profile_images/1908492682470780928/0DQPZ7YR.jpg",
    ).then((r) => r.arrayBuffer()),
  },
  {
    src: "large.jpg",
    data: await fetch(
      "https://pbs.twimg.com/media/G2okqu5bEAABOHF?format=jpg&name=large",
    ).then((r) => r.arrayBuffer()),
  },
];

export default function XPostImage() {
  return (
    <div
      style={{
        backgroundColor: "black",
        width: "100%",
        height: "100%",
        flexDirection: "column",
        padding: "3rem",
        paddingBottom: 0,
      }}
    >
      <div
        style={{
          marginBottom: "2rem",
          gap: "2rem",
          alignItems: "center",
        }}
      >
        <img
          src="fuma.jpg"
          alt="Fuma Nama"
          style={{
            width: 120,
            height: 120,
            borderRadius: "50%",
          }}
        />
        <div
          style={{
            flexDirection: "column",
            fontSize: "3rem",
            flexGrow: 1,
            gap: "0.5rem",
          }}
        >
          <span
            style={{
              color: "white",
              fontWeight: 700,
            }}
          >
            Fuma Nama
          </span>
          <span
            style={{
              marginTop: 0,
              color: "gray",
              fontWeight: 300,
            }}
          >
            @fuma_nama
          </span>
        </div>
        <img
          src="takumi.svg"
          alt="Takumi"
          style={{
            width: 64,
            height: 64,
          }}
        />
      </div>
      <span
        style={{
          lineClamp: 1,
          textOverflow: "ellipsis",
          fontSize: "4rem",
          color: "white",
          fontWeight: 300,
          marginBottom: "1rem",
        }}
      >
        My favourite part of the year
      </span>
      <div
        style={{
          width: "100%",
          flexGrow: 1,
        }}
      >
        <img
          src="large.jpg"
          style={{
            width: "100%",
            borderRadius: "2rem",
            borderWidth: 1,
            borderColor: "dimgray",
          }}
          alt="content"
        />
      </div>
      <div
        style={{
          position: "absolute",
          width: "100%",
          height: "100%",
          backgroundImage: "linear-gradient(to top, black, transparent 50%)",
        }}
      />
    </div>
  );
}
