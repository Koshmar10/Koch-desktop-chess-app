import { GameCreateResponse } from "../../api/bindings/GameCreateResponse";
import { GameResult } from "../../api/bindings/GameResult";

const RESULT_TOKEN: Record<GameResult, string> = {
  WhiteWin: "1-0",
  BlackWin: "0-1",
  Draw: "1/2-1/2",
  Unfinished: "*",
};

const pad = (n: number): string => n.toString().padStart(2, "0");

const today = (): string => {
  const now = new Date();
  return `${now.getFullYear()}.${pad(now.getMonth() + 1)}.${pad(now.getDate())}`;
};

const movetext = (moves: string[]): string =>
  moves
    .reduce<string[]>((pairs, move, idx) => {
      if (idx % 2 === 0) {
        pairs.push(`${idx / 2 + 1}. ${move}`);
      } else {
        pairs[pairs.length - 1] += ` ${move}`;
      }
      return pairs;
    }, [])
    .join(" ");

// Builds a PGN for a just-finished koch game. Everything a PGN needs —
// players, time control, move history, result — is already in
// GameCreateResponse, so this needs no round trip to the backend (unlike
// an imported game, whose pgn_data is the file it came from).
export const buildPgn = (game: GameCreateResponse): string => {
  const result = RESULT_TOKEN[game.state.result];
  const timeControl = `${Math.round(game.time_control.initial_ms / 1000)}+${Math.round(
    game.time_control.increment_ms / 1000,
  )}`;

  const tags = [
    `[Event "Casual Game"]`,
    `[Site "Koch"]`,
    `[Date "${today()}"]`,
    `[White "${game.white_player.name}"]`,
    `[Black "${game.black_player.name}"]`,
    `[WhiteElo "${game.white_player.elo}"]`,
    `[BlackElo "${game.black_player.elo}"]`,
    `[TimeControl "${timeControl}"]`,
    `[Result "${result}"]`,
  ].join("\n");

  const moves = movetext(game.state.move_history);
  const line = moves ? `${moves} ${result}` : result;

  return `${tags}\n\n${line}\n`;
};
