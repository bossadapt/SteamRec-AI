"use client";
import { useEffect, useState } from "react";
import Styles from "./page.module.css";
import { headers } from "next/headers";
export default function Home() {
  const [getLink, setLink] = useState("");
  function grabRecList() {
    console.log("attempting to grab rec list");
    api("http://127.0.0.1:5001/convert/" + getLink).then((result) => {
      console.log("result:");
      console.log(result);
    });
  }
  function api<T>(url: string): Promise<void | T> {
    return fetch(url)
      .then((response) => {
        if (!response.ok) {
          console.log("error happened");
          console.log(response);
          throw new Error(response.statusText);
        }
        console.log("got what it needed before json");
        return response.json() as Promise<T>;
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
          <button className={Styles.searchButton} onClick={() => grabRecList()}>
            Search
          </button>
        </div>
      </div>
    </div>
  );
}
