import React, { useEffect, useState, Fragment } from 'react';
import BoardFENWrapper from './BoardFENWrapper';
import BitboardInspect from './BitboardInspect';

const API_URL = 'ws://localhost:9001';
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

interface Player {
  id: number,
  name: string,
  apikey: string | null,
  wins: number,
  losses: number,
  ties: number,
  game_ids: number[],
  cur_game_index: number,
}

interface Game {
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

class GamesState {
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
      cur_game_index: Number(cmd[6]),
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
        players = {
          ...players,
          [id]: {
            ...players[id],
            apikey: key,
          }
        };
      } else {
        const player = {
          id: id,
          apikey: key,
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
  const [gamesState, setGamesState] = useState(new GamesState({}, {}, null));
  const [curApiKey, setCurApiKey] = useState<string | null>(null);
  const [apiLogin, setApiLogin] = useState<string>("");
  const [changeName, setChangeName] = useState<string>("");
  const [showBitboardConstruct, setShowBitboardConstruct] = useState(false);

  useEffect(() => {
    const newSocket = new WebSocket(API_URL);
    newSocket.addEventListener("open", (event) => {
      console.log("WebSocket opened");
      newSocket.send("observe");
    });
    newSocket.addEventListener("message", (event) => {
      handleServerMsg(event.data);
    });
    newSocket.addEventListener("error", (event) => {
      console.error("Websocket error: ", event);
    });
    setSocket(newSocket);
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
    setCurApiKey(key);
    socket?.send("apikey " + key);
    socket?.send("playerid");
  }

  const playersElem = [];
  for(const [id, player] of Object.entries(gamesState.players)) {
    playersElem.push(
      <tr key={id}>
        <td className="playerName">{player.name}</td>
        <td>{`${player.wins}-${player.losses}-${player.ties}`}</td>
      </tr>
    );
  }

  const players = (
    <table className="playersTable">
      <thead>
        <tr>
          <th>Name</th>
          <th>Score</th>
        </tr>
      </thead>
      <tbody>
        {playersElem}
      </tbody>
    </table>
  );

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

  const gamesElem: JSX.Element[] = [];
  games.forEach((game) => {
    const white = gamesState.players[game.white_id];
    const black = gamesState.players[game.black_id];
    const scoreStr = gamesState.gameScoreStr(game.id);
    let status = "Not Started";
    if(game.finished) {
      status = `Score: ${scoreStr}`;
    } else if(gamesState.gameIsActive(game.id)) {
      status = game.fen.split(" ")[1] === "w" ? "White To Move" : "Black To Move";
    }
    gamesElem.push(
      <div className="gameContainer" key={game.id} id={`game-${game.id}`}>
        <div className="gameTitleContainer flex">
          <div className="gameTitleLeft">
            <div>{white.name}</div>
            {white.apikey !== null &&
              <Fragment>
                <div className="apikey">API Key: {white.apikey}</div>
                { curApiKey !== white.apikey ?
                  <span 
                    className="playAs"
                    onClick={() => {playAs(white.apikey)}}
                  >Play as this player</span>
                  :
                  <div className="apikey">
                    Current Player
                  </div>
                }
              </Fragment>
            }
          </div>
          <div className="gameTitleCenter">
            vs
          </div>
          <div className="gameTitleRight">
            {black.name}
            {black.apikey !== null &&
              <Fragment>
                <div className="apikey">API Key: {black.apikey}</div>
                {curApiKey !== black.apikey ?
                  <span
                    className="playAs"
                    onClick={() => {playAs(black.apikey)}}
                  >Play as this player</span>
                  :
                  <div className="apikey">
                    Current Player
                  </div>
                }
              </Fragment>
            }
          </div>
        </div>
        <div className="flex">
          <div className="flexExpand"/>
          <div>
            <div className="flex">
              <div className="gameBody">
                <BoardFENWrapper 
                  fen={game.fen}
                  legal_moves={game.legal_moves}
                  size={400}
                  reversed={game.black_id === gamesState.curPlayerId}
                  allow_moves={[game.white_id === gamesState.curPlayerId, game.black_id === gamesState.curPlayerId]}
                  onMove={(move) => {
                    socket?.send("move " + move + "\n");
                  }}/>
              </div>
              <div className="gameMoves flex">
                <table className="gameMovesTable roundedDiv scroll">
                  <tbody>
                    {game.moves.filter((e, i) => i % 2 === 0).map((e, i) => (
                      <tr key={i}>
                        <td className="moveNum">{i + 1}.</td>
                        <td className="move">{game.moves[i * 2]}</td>
                        {game.moves.length > (i * 2 + 1) &&
                          <td className="move">{game.moves[i * 2 + 1]}</td>
                        }
                      </tr>
                    ))}
                  </tbody>
                </table>
                <div className="gameStatus roundedDiv">{status}</div>
              </div>
            </div>
            {game.client_data.map((data: {[key: string]: string}) => {
              const body = []
              for(const [key, value] of Object.entries(data)) {
                //if(key === "eval") continue;
                body.push(
                  <tr>
                    <td className="playerName">{key}</td>
                    <td>{value}</td>
                  </tr>
                );
              }
              return (
                <div className="flex gameInfo">
                  <div className="gameInfoDiv roundedDiv scroll">
                    <table>
                      <tbody>
                        {body}
                      </tbody>
                    </table>
                  </div>
                </div>
              );
            })}
          </div>
          <div className="flexExpand"/>
        </div>
      </div>
    );
  });

  const gamesSidebarElems = [];
  for(const [id, game] of Object.entries(gamesState.games)) {
    const white = gamesState.players[game.white_id];
    const black = gamesState.players[game.black_id];
    const statusMsg = game.finished ? gamesState.gameScoreStr(game.id) : gamesState.gameIsActive(game.id) ? "In Progress" : "No Started";
    gamesSidebarElems.push(
      <tr key={id}>
        <td className="playerName">{white.name} vs {black.name}</td>
        <td>{statusMsg}</td>
      </tr>
    );
  }

  const gamesSidebar = (
    <table className="playersTable">
      <thead>
        <tr>
          <th>Match</th>
          <th>Outcome</th>
        </tr>
      </thead>
      <tbody>
        {gamesSidebarElems}
      </tbody>
    </table>
  );

  const playerSidebar = (
    <Fragment>
      <div className="currentPlayer">
        {ALLOW_LOGIN &&
          <Fragment>
            <div className="playersFormLabel">Log In With API Key:</div>
            <form 
              className="flex formContainer"
              onSubmit={(event) => {
                playAs(apiLogin);
                setApiLogin("");
                event.preventDefault();
              }}
            >
              <input type="text" value={apiLogin} onChange={(event) => {setApiLogin(event.target.value)}}></input>
              <input type="submit" value="Go"></input>
            </form>
          </Fragment>
        }
        {gamesState.curPlayerId !== null &&
          <Fragment>
          <br/>
          Current Player: <br/>
            <div className="playersTitle">
              {gamesState.players[gamesState.curPlayerId].name}
            </div>
            <div className="playersFormLabel">Change Name:</div>
            <form 
              className="flex formContainer"
              onSubmit={(event) => {
                socket?.send("name " + changeName);
                setChangeName("");
                event.preventDefault();
              }}
            >
              <input type="text" value={changeName} onChange={(event) => {setChangeName(event.target.value)}}></input>
              <input type="submit" value="Go"></input>
            </form>
          </Fragment>
        }
        </div>
        <div className="playersTitle">Players</div>
        {players}
        <div className="playersTitle">Games</div>
        {gamesSidebar}
    </Fragment>
  );

  return (
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
            <button type="button" className="header-btn" onClick={() => {
              socket?.send("newgame");
            }}>New Game</button>
          }
          <button type="button" className="header-btn" onClick={() => {
            setShowBitboardConstruct(true);
          }}>Bitboard Construction Tool</button>
        </div>
        {gamesElem}
      </div>
      <div className="gameSidebarContainer">
        {playerSidebar}
      </div>
    </div>
  );

}