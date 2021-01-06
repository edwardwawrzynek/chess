import React, { useState } from 'react';
import Long from 'long';

export interface BitboardInspectProps {
  onClose: () => void
}

function parseNumber(num: string): Long {
  if(num.startsWith("0x") || num.startsWith("0X")) {
    if(num.length === 2) return Long.ZERO;

    return Long.fromString(num.substring(2), 16);
  }
  else if(num.startsWith("0b") || num.startsWith("0X")) {
    if(num.length === 2) return Long.ZERO;

    return Long.fromString(num.substring(2), 2);
  } else {
    return Long.fromString(num, 10);
  }
}

export default function BitboardInspect(props: BitboardInspectProps) {
  const [val, setVal] = useState(Long.ZERO);
  const [valStr, setValStr] = useState("0x0");

  function adjustValueAll(val: Long) {
    setVal(val);
    setValStr(`0x${val.toUnsigned().toString(16)}`);
  }

  function getCell(val: Long, x: number, y: number): number {
    const longVal = Long.fromValue(val);
    return (longVal.shiftRightUnsigned(x + y * 8)).and(1).toNumber();
  }

  function flipCell(val: Long, x: number, y: number): Long {
    return val.xor(Long.ONE.shiftLeft(x + y * 8));
  }

  return (
    <div className="bitboard-container">
      <div className="flex">
        <div className="flexExpand"></div>
        <div className="bitboard-box">
          <div 
            className="bitboard-close"
            onClick={() => {
              props.onClose();
            }}
          >
            X
          </div>
          <h2>Bitboard Construction Tool</h2>
          <p>Click cells to toggle, or enter a value.</p>
          <div className="flex">
            <div className="flexExpand"></div>
            <div className="bitboard">
              {
                Array(10).fill(0).map((_, y) => (
                  Array(10).fill(0).map((_, x) => {
                    let cell = " ";
                    let onBoard = false;
                    let hit = 0;
                    if((x === 0 || x === 9) && y > 0 && y < 9) {
                      cell = "87654321"[y - 1];
                    } else if((y === 0 || y === 9) && x > 0 && x < 9) {
                      cell = "abcdefgh"[x - 1];
                    } else if (x > 0 && x < 9 && y > 0 && y < 9) {
                      onBoard = true;
                      hit = getCell(val, x - 1, 8 - y);
                      cell = hit ? "1" : "0";
                    }
                    return (
                      <div 
                        className={`
                          bitboard-cell 
                          ${y === 0 || y === 9 || x === 0 || x === 9 ? 'bitboard-head' : ""}
                          ${onBoard && (hit ? "bitboard-high" : "bitboard-low")}
                        `}
                        style={{gridColumn: x + 1, gridRow: y + 1}}
                        onClick={() => {
                          if(x > 0 && x < 9 && y > 0 && y < 9) {
                            adjustValueAll(flipCell(val, x - 1, 8 - y));
                          }
                        }}>{cell}</div>
                    );
                  })
                ))
              }
            </div>
            <div className="flexExpand"></div>
          </div>
          <div>
            Value: 
            <input className="bitboard-val-input" type="text" value={valStr} onChange={(e) => {
              setValStr(e.target.value);
              setVal(parseNumber(e.target.value));
            }}></input>
          </div>
        </div>
        <div className="flexExpand"></div>
      </div>
    </div>
  )
}