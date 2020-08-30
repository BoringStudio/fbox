import React from 'react';

import { State, StateContext } from './state';
import { ConnectionPage } from './pages/Connection';
import { LoadingPage } from './pages/Loading';
import { MainPage } from './pages/Main';

const App = () => {
  return (
    <State>
      <StateContext.Consumer>
        {({ kind }) =>
          //<MainPage />
          (kind === 'uninitialized' && <LoadingPage />) ||
          (kind === 'created' && <ConnectionPage />) ||
          (kind === 'connected' && <MainPage />)
        }
      </StateContext.Consumer>
    </State>
  );
};

export default App;
