import React from 'react';
import { BrowserRouter as Router, Switch, Route } from 'react-router-dom';

import { MainPage } from './pages/Main';
import { WebSocketService } from './components/WebSocketService';

const App = () => {
  return <>
    <WebSocketService />
    <Router>
      <Switch>
        <Route path="/">
          <MainPage />
        </Route>
      </Switch>
    </Router>
  </>;
};

export default App;
