import React from 'react';
import { BrowserRouter as Router, Switch, Route } from 'react-router-dom';

import { State } from './state';
import { MainPage } from './pages/Main';

const App = () => {
  return (
    <State>
      <Router>
        <Switch>
          <Route path="/">
            <MainPage />
          </Route>
        </Switch>
      </Router>
    </State>
  );
};

export default App;
