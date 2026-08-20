import Chessboard from "../../components/chessboard/Chessboard";
import Squares from "../../components/chessboard/layers/Squares";
import PieceLayer from "../../components/chessboard/layers/PieceLayer";
import ArrowLayer from "../../components/chessboard/layers/ArrowLayer";
import { PlayerCard } from "../../components/chessboard/PlayerCard";
import PlayControls from "./PlayControls";
import {
  MOCK_BLACK_CAPTURES,
  MOCK_BLACK_CLOCK,
  MOCK_WHITE_CAPTURES,
  MOCK_WHITE_CLOCK,
} from "./mock";

const Play = () => {
  return (
    <div className="flex flex-col justify-center items-center w-full h-full">
      <div className="flex flex-col gap-2 w-full max-w-[576px]">
        <PlayerCard
          display={true}
          color="black"
          player="Black"
          piecesTaken={MOCK_BLACK_CAPTURES}
          clock={MOCK_BLACK_CLOCK}
          isTurn={false}
        />
        <Chessboard>
          <Squares />
          <PieceLayer />
          <ArrowLayer />
        </Chessboard>
        <PlayerCard
          display={true}
          color="white"
          player="White"
          piecesTaken={MOCK_WHITE_CAPTURES}
          clock={MOCK_WHITE_CLOCK}
          isTurn={true}
        />
      </div>
      <PlayControls />
    </div>
  );
};

export default Play;
