"use client";
import { useEffect, useState } from "react";
import Styles from "./page.module.css";
import { headers } from "next/headers";
interface APIOutput {
  success: boolean;
  error: String;
  games_included: boolean;
  reviews_included: boolean;
  games: Game[];
}
interface Game {
  name: String;
  steam_appid: number;
  score: number;
  is_free: boolean;
  detailed_description: String;
  developers: String[] | null;
  capsule_image: String;
  release_date: ReleaseDate;
  platforms: Platforms;
  price_overview: PriceOverview | null;
  content_descriptors: ContentDescriptors;
}
interface PriceOverview {
  final_formatted: String;
}
interface ContentDescriptors {
  ids: number[];
  notes: String | null;
}
interface ReleaseDate {
  coming_soon: boolean;
  date: String;
}
interface Platforms {
  windows: boolean;
  mac: boolean;
  linux: boolean;
}
export default function Home() {
  const [getLink, setLink] = useState("");
  const [getSearchButtionDisabled, setSearchButtionDisabled] = useState(false);
  const validProfile = new RegExp(
    "(https://)?steamcommunity.com/profiles/[0-9]{17}$"
  );
  const validID = new RegExp(
    "(https://)?steamcommunity.com/id/[a-zA-Z0-9]{1,32}$"
  );
  function grabRecList() {
    setSearchButtionDisabled(true);
    console.log("attempting to grab rec list");
    if (validProfile.test(getLink)) {
      api(
        "http://127.0.0.1:5001/convert/profiles/" +
          getLink.substring(getLink.lastIndexOf("/") + 1, getLink.length)
      ).then((result) => {
        console.log("result:");
        console.log(result);
      });
    } else if (validID.test(getLink)) {
      api(
        "http://127.0.0.1:5001/convert/id/" +
          getLink.substring(getLink.lastIndexOf("/") + 1, getLink.length)
      ).then((result) => {
        console.log("result:");
        console.log(result);
      });
    } else {
      console.log(
        'link given should look like: "https://steamcommunity.com/profiles/71247192837400923" or "https://steamcommunity.com/id/coolName1234$"'
      );
    }
    setSearchButtionDisabled(false);
  }
  function api<APIOutput>(url: string): Promise<APIOutput | void> {
    return fetch(url)
      .then((response) => {
        if (!response.ok) {
          console.log("error happened");
          console.log(response);
          throw new Error(response.statusText);
        }
        console.log("got what it needed before json");
        return response.json() as Promise<APIOutput>;
      })
      .catch((err) => {
        console.log(err);
      });
  }
  return (
    <div style={{ display: "flex" }}>
      <div style={{ display: "flex", flexDirection: "column", width: "100%" }}>
        <h1 className={Styles.title}>SteamRec AI</h1>
        <h2 className={Styles.nameEntryTitle}>Steam Profile Link</h2>
        <div style={{ display: "flex", flexDirection: "row" }}>
          <input
            className={Styles.accountEntry}
            placeholder="https://steamcommunity.com/id/AbacusAvenger"
            value={getLink}
            onChange={(eve) => {
              setLink(eve.target.value);
            }}
          ></input>
          <button
            className={Styles.searchButton}
            disabled={getSearchButtionDisabled}
            onClick={() => grabRecList()}
          >
            Search
          </button>
        </div>
      </div>
    </div>
  );
}
