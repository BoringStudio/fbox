import React from 'react';

import { StateContext, IStateContext } from '../../state';
import { FileInput } from '../../components/FileInput';

import './style.scss';

const LOCALIZATION = {
  addPeerButton: 'Add Peer',
  disconnectButton: 'Disconnect'
};

export const MainPage = () => {
  const session = React.useContext(StateContext) as IStateContext<'connected'>;

  const [peerMnemonics, setPeerMnemonics] = React.useState('');

  const onFilesAdded = (files: File[]) => files.forEach(file => session.addFile(file));

  return (
    <div className="content main-page">
      <ul>
        {session.files.map(file => (
          <li>
            <a
              className="button"
              style={{ marginBottom: '1em' }}
              href={new URL(
                `/v1/sessions/files/${file.id}?session_seed=${session.seed}`,
                process.env.REACT_APP_API_URL
              ).toString()}
            >
              <pre>{JSON.stringify(file)}</pre>
            </a>
          </li>
        ))}
      </ul>
      <FileInput onDrop={onFilesAdded} />
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
