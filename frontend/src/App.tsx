import React from 'react';

import { State, StateContext } from './state';
import { MainPage } from './pages/Main';
import { LoadingPage } from './pages/Loading';

const App = () => {
  return (
    <State>
      <StateContext.Consumer>
        {({ kind }) =>
          (kind === 'uninitialized' && <LoadingPage />) ||
          (kind === 'created' && <MainPage />)
        }
      </StateContext.Consumer>
    </State>
  );
};

export default App;
