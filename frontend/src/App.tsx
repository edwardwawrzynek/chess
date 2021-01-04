import React, { useState, useEffect, Fragment } from 'react';
import './App.css';
import Game from './Game';
import Games from './Games';
import clibModule from './clib/interface';

function App() {
  const [clib, setClib] = useState<any>(null);

  useEffect(() => {
    clibModule().then((module: any) => {
      setClib(module);
    });
  }, []);

  return (
    <div className="App">
      {clib === null &&
        <p>Loading WebAssembly modules...</p>
      }
      {clib !== null &&
        <Fragment>
          <Games clib={clib}></Games>
        </Fragment>
      }
    </div>
  );
}

export default App;
