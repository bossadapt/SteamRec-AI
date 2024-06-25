import { LoadingState } from "./globalInterfaces";
import Styles from "./page.module.css";
interface LoadingProps {
  state: LoadingState;
  error: String;
}
export const Loading: React.FC<LoadingProps> = ({ state, error }) => {
  console.log("reached loading");
  function getLoaderHidden(): boolean {
    if (state == LoadingState.Failed || state == LoadingState.WaitingForLink) {
      return true;
    }
    return false;
  }
  function getMessageColor(): string {
    if (state == LoadingState.Failed) {
      return "red";
    } else if (state == LoadingState.Finished) {
      return "green";
    } else {
      return "white";
    }
  }
  function getLoadingSymbol(): JSX.Element {
    if (
      state == LoadingState.WaitingForLink ||
      state == LoadingState.Failed ||
      state == LoadingState.Finished
    ) {
      return <div></div>;
    } else {
      return <span className={Styles.loader}></span>;
    }
  }
  return (
    <div
      style={{
        width: "100%",
        height: "100%",
        marginLeft: "auto",
        marginRight: "auto",
        textAlign: "center",
      }}
    >
      {getLoadingSymbol()}
    </div>
  );
};
export default Loading;
