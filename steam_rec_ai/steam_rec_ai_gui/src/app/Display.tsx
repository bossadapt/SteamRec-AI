import { useState } from "react";
import {
  APIOutput,
  Game,
  PriceOverview,
  ContentDescriptors,
  ReleaseDate,
  Platforms,
} from "./globalInterfaces";
interface DisplayProps {
  input: APIOutput;
}

export const Display: React.FC<DisplayProps> = ({ input }) => {
  console.log("reached display");
  const [showNudity, setShowNudity] = useState(false);
  const [showOnlyFree, setShowOnlyFree] = useState(false);
  const [showDeletedGames, setShowDeletedGames] = useState(false);
  const [currentPage, setCurrentPage] = useState(0);
  let fullListToShow = input.games.filter((e) => {
    if (!showDeletedGames && e.name == "unknown") {
      return false;
    }
    if (!showNudity && e.content_descriptors.ids.includes(1)) {
      return false;
    }
    if (showOnlyFree) {
      return e.is_free;
    } else {
      return true;
    }
  });

  let shownList = fullListToShow
    .slice(currentPage, currentPage + 5)
    .map((game) => {
      return (
        <a
          target="_blank"
          href={"https://store.steampowered.com/app/" + game.steam_appid}
          style={{ textDecoration: "none" }}
        >
          <div
            style={{
              borderColor: "#ff3d00",
              border: "2px ,#ff3d00",
              borderStyle: "solid",
              display: "flex",
              flexDirection: "row",
              marginBottom: "10px",
            }}
          >
            <img
              src={game.header_image.toString()}
              style={{ width: "25%", marginRight: "1%" }}
            ></img>
            <div
              style={{
                display: "flex",
                flexDirection: "column",
                width: "75%",
              }}
            >
              <div
                style={{
                  display: "flex",
                  flexDirection: "row",
                  height: "25%",
                  width: "100%",
                }}
              >
                <h3 style={{ marginLeft: "0px" }}>
                  {game.name == "unknown"
                    ? "appid:" + game.steam_appid
                    : game.name}
                </h3>
                <h3 style={{ marginRight: "10px", marginLeft: "auto" }}>
                  {game.is_free
                    ? "Free"
                    : game.price_overview?.final_formatted || "N/A"}
                </h3>
              </div>
              <div
                style={{
                  display: "flex",
                  flexDirection: "row",
                  height: "75%",
                  width: "100%",
                }}
              >
                <p>
                  {game.name == "unknown"
                    ? "Game removed from steam"
                    : game.short_description}
                </p>
              </div>
            </div>
          </div>
        </a>
      );
    });
  let prevButtonDisabled = currentPage == 0;
  let nextButtonDisabled = currentPage + 5 > fullListToShow.length;
  function getGamesColor(): string {
    if (input.games_included) {
      return "green";
    } else {
      return "red";
    }
  }
  function nextButtonPressed() {
    setCurrentPage((currentPage) => {
      console.log(fullListToShow.length);
      return currentPage + 5;
    });
  }
  function prevButtonPressed() {
    setCurrentPage((currentPage) => {
      return currentPage - 5;
    });
  }
  function getReviewsColor(): string {
    if (input.reviews_included) {
      return "green";
    } else {
      return "red";
    }
  }
  return (
    <div style={{ display: "flex", flexDirection: "column" }}>
      <div
        style={{
          display: "flex",
          flexDirection: "row",
        }}
      >
        <button
          disabled={prevButtonDisabled}
          style={{
            width: "10%",
            marginRight: "5%",
            color: "black",
            fontSize: "20px",
            border: "0px",
            backgroundColor: prevButtonDisabled ? "black" : "#ff3d00",
          }}
          onClick={() => prevButtonPressed()}
        >
          {"< "}Previous
        </button>
        <h2 style={{ marginLeft: "0px", color: getGamesColor() }}>
          Games Included
        </h2>
        <h2 style={{ marginLeft: "3%", color: getReviewsColor() }}>
          Reviews Included
        </h2>

        <h2 style={{ marginLeft: "auto", marginRight: "1%" }}>
          Show Deleted Games
        </h2>
        <input
          style={{ marginRight: "3%", color: "white", width: "1.5%" }}
          checked={showDeletedGames}
          onChange={() =>
            setShowDeletedGames((e) => {
              return !e;
            })
          }
          type="checkbox"
        />

        <h2 style={{ marginLeft: "auto", marginRight: "1%" }}>
          Only Free Games
        </h2>
        <input
          style={{ marginRight: "3%", color: "white", width: "1.5%" }}
          checked={showOnlyFree}
          onChange={() =>
            setShowOnlyFree((e) => {
              return !e;
            })
          }
          type="checkbox"
        />
        <h2 style={{ marginLeft: "auto", marginRight: "1%" }}>Show Nudity</h2>

        <input
          style={{ marginRight: "3%", color: "white", width: "1.5%" }}
          checked={showNudity}
          onChange={() =>
            setShowNudity((e) => {
              return !e;
            })
          }
          type="checkbox"
        />
        <button
          disabled={nextButtonDisabled}
          style={{
            width: "10%",
            marginLeft: "5%",
            color: "black",
            fontSize: "20px",
            border: "0px",
            backgroundColor: nextButtonDisabled ? "black" : "#ff3d00",
          }}
          onClick={() => nextButtonPressed()}
        >
          Next{" >"}
        </button>
      </div>
      <div>{shownList}</div>
    </div>
  );
};
export default Display;
