"use client";
import { useEffect, useState } from "react";
import Styles from "./page.module.css";
import { headers } from "next/headers";
import Loading from "./Loading";
import Display from "./Display";
import { APIOutput, Game, LoadingState } from "./globalInterfaces";
export default function Home() {
  const [link, setLink] = useState("");
  const [output, setOutput] = useState<APIOutput>({
    success: false,
    error: "",
    games_included: false,
    reviews_included: false,
    games: [],
  });
  const [errorMessage, setErrorMessage] = useState("");
  const [currentLoadingState, SetCurrentLoadingState] = useState(
    LoadingState.WaitingForLink
  );
  const [searchButtionDisabled, setSearchButtionDisabled] = useState(false);
  const validProfile = new RegExp(
    "(https://)?steamcommunity.com/profiles/[0-9]{17}$"
  );
  const validID = new RegExp(
    "(https://)?steamcommunity.com/id/[a-zA-Z0-9]{1,32}$"
  );
  function BodyOutput(): JSX.Element {
    if (currentLoadingState == LoadingState.WaitingForLink) {
      return <div></div>;
    }
    if (
      currentLoadingState != LoadingState.Finished &&
      output.games.length == 0
    ) {
      console.log("set to loading");
      return (
        <Loading error={errorMessage} state={currentLoadingState}></Loading>
      );
    } else {
      console.log("set to display");
      return <Display input={output}></Display>;
    }
  }
  function getMessageColor(): string {
    if (currentLoadingState == LoadingState.Failed) {
      return "red";
    } else if (currentLoadingState == LoadingState.Finished) {
      return "green";
    } else {
      return "white";
    }
  }
  function getMessage() {
    switch (currentLoadingState) {
      case LoadingState.WaitingForLink: {
        console.log("WaitingForLink");
        return 'Please enter the link for your profile that should look like "https://steamcommunity.com/id/customname" or "https://steamcommunity.com/profiles/70000000000000000"';
      }
      case LoadingState.VerifyingLink: {
        console.log("VerifyingLink");
        return "checking if link given is valid";
      }
      case LoadingState.ScrapingAndGuessing: {
        console.log("ScrapingAndGuessing");
        return "Attempting to scrape profile, convert to a scoring system and finally run it through an AI model based on over 600,000 public profiles";
      }
      case LoadingState.SortingOutput: {
        console.log("SortingOutput");
        return "Recieved a list of games scoring, Sorting it now";
      }
      case LoadingState.Finished: {
        console.log("Finished");
        return "All good to go, Displaying now";
      }
      case LoadingState.Failed: {
        console.log("Failed");
        return "ERROR: " + errorMessage;
      }
    }
  }
  function grabRecList() {
    setSearchButtionDisabled(true);
    SetCurrentLoadingState(LoadingState.VerifyingLink);
    let currentLink = link;
    if (currentLink.lastIndexOf("/") == currentLink.length - 1) {
      currentLink = currentLink.substring(0, currentLink.length - 1);
    }
    if (validProfile.test(currentLink)) {
      SetCurrentLoadingState(LoadingState.ScrapingAndGuessing);
      api(
        "http://127.0.0.1:5001/convert/profiles/" +
          currentLink.substring(
            currentLink.lastIndexOf("/") + 1,
            currentLink.length
          )
      ).then((result) => {
        if (result.success) {
          handleGoodAPIReturn(result);
        } else {
          console.log("error reached");
          SetCurrentLoadingState(LoadingState.Failed);
          setErrorMessage(result.error.toString());
        }
      });
    } else if (validID.test(currentLink)) {
      SetCurrentLoadingState(LoadingState.ScrapingAndGuessing);
      api(
        "http://127.0.0.1:5001/convert/id/" +
          currentLink.substring(
            currentLink.lastIndexOf("/") + 1,
            currentLink.length
          )
      ).then((result) => {
        if (result.success) {
          console.log(result);
          handleGoodAPIReturn(result);
        } else {
          console.log("error reached");
          SetCurrentLoadingState(LoadingState.Failed);
          setErrorMessage(result.error.toString());
        }
      });
    } else {
      SetCurrentLoadingState(LoadingState.Failed);
      setErrorMessage(
        'link failed validation, should look like: "https://steamcommunity.com/profiles/71247192837400923" or "https://steamcommunity.com/id/coolName1234"'
      );
    }
    console.log("state:", currentLoadingState);
    console.log("error:", errorMessage);
    setSearchButtionDisabled(false);
  }
  ///sorting the output and switching from loading component to display component
  function handleGoodAPIReturn(output_given: APIOutput) {
    SetCurrentLoadingState(LoadingState.SortingOutput);
    output_given.games = output_given.games.sort((a, b) => b.score - a.score);
    setOutput(output_given);
    SetCurrentLoadingState(LoadingState.Finished);
  }
  async function api(url: string): Promise<APIOutput> {
    return await fetch(url).then(async (response) => {
      if (!response.ok) {
        return {
          success: false,
          error: "Failed reaching the API",
          games_included: false,
          reviews_included: false,
          games: [] as Game[],
        } as APIOutput;
      }
      let test: APIOutput = await Promise.resolve(response.json()).then(
        (x) => x as APIOutput
      );
      return test;
    });
  }
  return (
    <div style={{ display: "flex" }}>
      <div style={{ display: "flex", flexDirection: "column", width: "100%" }}>
        <div style={{ display: "flex", flexDirection: "row" }}>
          <h1 className={Styles.title1}>SteamRec</h1>
          <h1 className={Styles.title2}>AI</h1>
        </div>
        <h2 className={Styles.nameEntryTitle}>Steam Profile Link</h2>
        <div style={{ display: "flex", flexDirection: "row" }}>
          <input
            className={Styles.accountEntry}
            placeholder="https://steamcommunity.com/id/AbacusAvenger"
            value={link}
            onChange={(eve) => {
              setLink(eve.target.value);
            }}
          ></input>
          <button
            className={Styles.searchButton}
            disabled={searchButtionDisabled}
            onClick={() => grabRecList()}
          >
            Search
          </button>
        </div>
        <h3 style={{ color: getMessageColor(), textAlign: "center" }}>
          {getMessage()}
        </h3>
        <div>
          <BodyOutput></BodyOutput>
        </div>
      </div>
    </div>
  );
}
