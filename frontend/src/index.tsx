import React from 'react';
import ReactDOM from 'react-dom';
import axios from 'axios';

import './styles/main.scss';

import App from './App';

axios.defaults.baseURL = process.env.REACT_APP_API_URL;

ReactDOM.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
  document.getElementById('root')
);
