import React, { useState } from 'react';
import { GamesState } from './Games';

export interface UserControlsProps {
  state: GamesState,
  onSetKey: (key: string) => void,
  onSetName: (name: string) => void,
  allowLogin: boolean,
}

export default function UserControls(props: UserControlsProps) {
  const [curKey, setCurKey] = useState<string>("");
  const [name, setName] = useState<string>("");

  return (
    <>
      {props.allowLogin &&
        <>
          <div className="playersFormLabel">Log In With API Key:</div>
            <form 
              className="flex formContainer"
              onSubmit={(event) => {
                props.onSetKey(curKey);
                setCurKey("");
                event.preventDefault();
              }}
            >
              <input type="text" value={curKey} onChange={(event) => {setCurKey(event.target.value)}}></input>
              <input type="submit" value="Go"></input>
            </form>
        </>
      }
      {
        props.state.curPlayerId !== null && 
        <>
          <br/>
          Current Player: <br/>
          <div className="playersTitle">
              {props.state.players[props.state.curPlayerId].name}
            </div>
            <div className="playersFormLabel">Change Name:</div>
            <form 
              className="flex formContainer"
              onSubmit={(event) => {
                props.onSetName(name);
                setName("");
                event.preventDefault();
              }}
            >
              <input type="text" value={name} onChange={(event) => {setName(event.target.value)}}></input>
              <input type="submit" value="Go"></input>
            </form>
        </>
      }
    </>
  );
}