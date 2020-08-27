import React from 'react';

import { StateContext, IStateConnected } from '../../state';

import './style.scss';

const LOCALIZATION = {
  addPeerButton: 'Add Peer',
  disconnectButton: 'Disconnect'
};

export const MainPage = () => {
  const session = React.useContext(StateContext) as IStateConnected;

  const [peerMnemonics, setPeerMnemonics] = React.useState('');

  return (
    <div className="content main-page">
      <input
        className="session-code-input input"
        type="text"
        value={peerMnemonics}
        onChange={event => setPeerMnemonics(event.currentTarget.value)}
      />
      <br />
      <button
        className="button"
        onClick={() => {
          session.addPeer(peerMnemonics);
          setPeerMnemonics('');
        }}
      >
        {LOCALIZATION.addPeerButton}
      </button>
      <div className="seed">{session.seed}</div>
      <button className="button" onClick={session.reconnect}>
        {LOCALIZATION.disconnectButton}
      </button>
    </div>
  );
};
