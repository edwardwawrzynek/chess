import React, { useEffect, useState, Fragment } from 'react';
import BoardFENWrapper from './BoardFENWrapper';
import BitboardInspect from './BitboardInspect';
import EvalGraph from './EvalGraph';
import UserControls from './UserControls';
import PlayersTable from './PlayersTable';
import GameUI from './GameUI';
import Select from 'react-select';
import { selectStyles } from './selectStyles';

//const API_URL = process.env.NODE_ENV === "development" ? 'ws://localhost:9001' : 'wss://codekata-chess.herokuapp.com';
const API_URL = 'wss://codekata-chess.herokuapp.com';
const TOURNAMENT_GAME_ORDERING = false;
const ALLOW_NEW_GAME = true;
const ALLOW_LOGIN = true;

type Command = string[];

// parse commands from the server
function parseCommand(msg: string): Command[] {
  const lines = msg.trim().split("\n");
  const cmds = lines.map((line) => {
    if(line.indexOf(' ') === -1)
      return [line];
    else
      return [
        line.substring(0, line.indexOf(' ')), 
        line.substring(line.indexOf(' ') + 1)
      ];
  });
  const cmdsAndArgs = cmds.map((cmd) => {
    if(cmd.length <= 1) {
      return cmd;
    } else {
      return [
        cmd[0], 
        ...cmd[1].split(",").map((c) => c.trim())
      ];
    }
  });

  return cmdsAndArgs;
}

export interface Player {
  id: number,
  name: string,
  apikey: string | null,
  wins: number,
  losses: number,
  ties: number,
  game_ids: number[],
  cur_game_index: number,
}

export interface Game {
  id: number,
  white_id: number,
  black_id: number,
  fen: string,
  moves: string[],
  finished: boolean,
  score: number,
  legal_moves: string[],
  client_data: [{[key: string]: string}, {[key: string]: string}]
}

export class GamesState {
  players: {[key: number]: Player};
  games: {[key: number]: Game};
  curPlayerId: number | null;

  constructor(players: {[key: number]: Player}, games: {[key: number]: Game}, curPlayerId: number | null) {
    this.players = players;
    this.games = games;
    this.curPlayerId = curPlayerId;
  }

  private runCmdPlayer(cmd: Command): GamesState {
    const id = Number(cmd[1]);
    let player = {
      id: id,
      apikey: cmd[2] !== "-" ? cmd[2] : null,
      name: cmd[3],
      wins: Number(cmd[4]),
      losses: Number(cmd[5]),
      ties: Number(cmd[6]),
      game_ids: cmd[7].split(" ").map((c) => Number(c)),
      cur_game_index: Number(cmd[8]),
    };
    if(player.apikey === null && id in this.players && this.players[id].apikey !== null) {
      player = {
        ...player,
        apikey: this.players[id].apikey,
      };
    }

    return new GamesState(
      {
        ...this.players,
        [id]: player,
      },
      this.games,
      this.curPlayerId,
    );
  }

  private parseClientData(str: string): {[key: string]: string} {
    let res: {[key: string]: string} = {};
    str.split("`").forEach((sec) => {
      const key = sec.split(" ")[0];
      const value = sec.substr(sec.indexOf(" ") + 1);
      res[key] = value;
    });
    return res;
  }

  private runCmdGame(cmd: Command, clib: any): GamesState {
    const id = Number(cmd[1]);
    // find legal moves
    // TODO: only find legal moves if required by curPlayerId
    const fen = cmd[4];
    const legal_moves = clib.ccall("clib_board_legal_moves", "string", ["string"], [fen]).split(",");
    let game: Game = {
      id: id,
      white_id: Number(cmd[2]),
      black_id: Number(cmd[3]),
      fen: fen,
      moves: cmd[5].split(" "),
      legal_moves: legal_moves,
      finished: cmd[6] === "1" ? true : false,
      score: cmd.length >= 7 ? Number(cmd[7]) : 0,
      client_data: [this.parseClientData(cmd[8]), this.parseClientData(cmd[9])]
    };

    return new GamesState(
      this.players,
      {
        ...this.games,
        [id]: game,
      },
      this.curPlayerId,
    );
  }

  private runCmdNewGame(cmd: Command): GamesState {
    let players = {...this.players};
    const cmd_players: [number, string][] = [[Number(cmd[2]), cmd[3]], [Number(cmd[4]), cmd[5]]];
    cmd_players.forEach(([id, key]) => {
      if(id in this.players) {
        let player = {
          ...players[id],
          apikey: key !== "-" ? key : null,
        };
        if(player.apikey === null && id in this.players && this.players[id].apikey !== null) {
          player = {
            ...player,
            apikey: this.players[id].apikey,
          };
        }

        players = {
          ...players,
          [id]: player
        };
      } else {
        const player = {
          id: id,
          apikey: key !== "-" ? key : null,
          name: "",
          wins: 0,
          losses: 0,
          ties: 0,
          game_ids: [],
          cur_game_index: 0,
        };
        players = {
          ...players,
          [id]: player,
        };
      }
    });
    return new GamesState(players, this.games, this.curPlayerId);
  }

  private runPlayerId(cmd: Command): GamesState {
    return new GamesState(this.players, this.games, Number(cmd[1]));
  }

  runCommand(cmd: Command, clib: any): GamesState {
    if(cmd[0] === "player") {
      return this.runCmdPlayer(cmd);
    } else if(cmd[0] === "game") {
      return this.runCmdGame(cmd, clib);
    } else if(cmd[0] === "newgame") {
      return this.runCmdNewGame(cmd);
    } else if(cmd[0] === "playerid") {
      return this.runPlayerId(cmd);
    } else if(cmd[0] === "error") {
      window.alert("error from server: " + cmd[1]);
      return this;
    } else {
      return this;
    }
  }

  gameIsActive(id: number): boolean {
    const game = this.games[id];
    const white = this.players[game.white_id];
    const black = this.players[game.black_id];
    if(game.finished) {
      return false;
    } else if(white.cur_game_index < white.game_ids.length && white.game_ids[white.cur_game_index] === game.id && black.cur_game_index < white.game_ids.length && white.game_ids[white.cur_game_index] === game.id) {
      return true;
    } else {
      return false;
    }
  }

  gameScoreStr(id: number): string {
    const game = this.games[id];
    return game.score === 0 ? ("1/2 - 1/2") : game.score === 1 ? "1 - 0" : "0 - 1";
  }
}

export interface GamesProps {
  clib: any,
}

export default function Games(props: GamesProps) {
  const [socket, setSocket] = useState<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);
  const [connErr, setConnErr] = useState<string | null>(null);
  const [gamesState, setGamesState] = useState(new GamesState({}, {}, null));
  const [showBitboardConstruct, setShowBitboardConstruct] = useState(false);

  const [newgamePlayer0, setNewgamePlayer0] = useState("-");
  const [newgamePlayer1, setNewgamePlayer1] = useState("-");

  useEffect(() => {
    const newSocket = new WebSocket(API_URL);
    newSocket.addEventListener("open", (event) => {
      newSocket.send("observe");
      setConnected(true);
    });
    newSocket.addEventListener("message", (event) => {
      handleServerMsg(event.data);
    });
    newSocket.addEventListener("error", (event) => {
      console.error("Websocket error: ", event);
      setConnErr(event.type);
    });
    setSocket(newSocket);
    const intervalId = setInterval(() => {
      newSocket.send("ping");
    }, 15000);

    return () => {
      clearInterval(intervalId);
    }
  }, []);

  function handleServerMsg(msg: string) {
    const cmds = parseCommand(msg);
    setGamesState((gamesState) => {
      let newState = gamesState;
      cmds.forEach((cmd) => {
        newState = newState.runCommand(cmd, props.clib);
      });
      return newState;
    });
  }

  function playAs(key: string | null) {
    if(key === null) {
      return;
    }
    socket?.send("apikey " + key);
    socket?.send("playerid");
  }

  function setName(name: string) {
    socket?.send("name " + name);
  }

  function makeMove(move: string) {
    socket?.send("move " + move);
  }

  function newgame() {
    socket?.send(`newgame ${newgamePlayer0}, ${newgamePlayer1}`)
  }

  // order game by id decreasing
  let gamesIDDec = [];
  for(const [id, game] of Object.entries(gamesState.games)) {
    gamesIDDec.push(game);
  }
  gamesIDDec.sort((a, b) => b.id - a.id);
  // sort games by active -> finished -> waiting
  let games: Game[] = [];
  if(TOURNAMENT_GAME_ORDERING) {
    gamesIDDec.forEach((game) => {
      if(gamesState.gameIsActive(game.id))
        games.push(game);
    });
    gamesIDDec.forEach((game) => {
      if(game.finished)
        games.push(game);
    });
    gamesIDDec.forEach((game) => {
      if(!gamesState.gameIsActive(game.id) && !game.finished)
        games.push(game);
    });
  } else {
    games = gamesIDDec;
  }

  const playerSidebar = (
    <>
      <div className="currentPlayer">
        <UserControls
          state={gamesState}
          allowLogin={ALLOW_LOGIN}
          onSetKey={(key: string) => playAs(key)}
          onSetName={(name: string) => setName(name)}
        />
      </div>
      <PlayersTable
        state={gamesState}
      />
    </>
  );

  let playerSelectOptions = [{value: '-', label: 'New User'}];
  for(const [id, player] of Object.entries(gamesState.players)) {
    playerSelectOptions.push({value: player.id.toString(), label: player.name});
  }

  const content = (
    <div className="gamesRootContainer flex">
      {showBitboardConstruct && 
        <BitboardInspect 
          onClose={() => {setShowBitboardConstruct(false);}}></BitboardInspect>
      }
      <div className="gamesContainer">
        <div className="header flex">
          <div className="headerTitle">Codekata Chess</div>
        </div>
        <div className="header-btns">
          {ALLOW_NEW_GAME && 
            <>
              New game between 
              <Select
                styles={selectStyles}
                className="playerSelect"
                options={playerSelectOptions}
                value={playerSelectOptions.find(option => option.value === newgamePlayer0)}
                onChange={(option) => {
                  setNewgamePlayer0(option!!.value);
                }}
              />and
              <Select
                styles={selectStyles}
                className="playerSelect"
                options={playerSelectOptions}
                value={playerSelectOptions.find(option => option.value === newgamePlayer1)}
                onChange={(option) => {
                  setNewgamePlayer1(option!!.value);
                }}
              /> 
              <button type="button" className="header-btn newgame-btn" onClick={newgame}>Create</button>
            </>
          }
        </div>
        <div className="header-btns">
        <button type="button" className="header-btn" onClick={() => {
            setShowBitboardConstruct(true);
          }}>Bitboard Construction Tool</button>
        </div>
        {
          games.map((game) => 
            <GameUI
              key={game.id}
              state={gamesState}
              gameId={game.id}
              onSetKey={(key: string | null) => playAs(key)}
              onMakeMove={(move: string) => makeMove(move)}
            />
          )
        }
      </div>
      <div className="gameSidebarContainer">
        {playerSidebar}
      </div>
    </div>
  );

  return (
    <>
      {!connected && connErr === null &&
        <p>Connecting to Server ...</p>
      }
      {connErr !== null &&
        <p>Error connecting to server: {connErr}</p>
      }
      {connected &&
        content
      }
    </>
  );

}