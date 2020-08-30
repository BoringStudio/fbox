import React from 'react';

import { v4 as uuidv4 } from 'uuid';

import { StateContext, IStateContext } from '../../state';
import { FileInput } from './FileInput';
import { FileButton } from './FileButton';

import './style.scss';

const LOCALIZATION = {
  addPeerButton: 'Add Peer'
};

export const MainPage = () => {
  const session = React.useContext(StateContext) as IStateContext<'connected'>;
  // const session: IStateContext<'connected'> = {
  //   kind: 'connected',
  //   connection_id: 1,
  //   seed: 'some seed',
  //   files: [
  //     {
  //       id: uuidv4(),
  //       name: 'My super file.txt',
  //       size: 123123123,
  //       mime_type: '',
  //       connection_id: 1
  //     }
  //   ],
  //   localFiles: new Map<string, File>(),
  //   addFile: (file: File) => console.log(file),
  //   removeFile: (id: string) => console.log(id),
  //   addPeer: (phrase: string) => console.log(phrase),
  //   reconnect: () => console.log('reconnect'),
  //   downloadFile: id => console.log(id)
  // };

  const [peerMnemonics, setPeerMnemonics] = React.useState('');

  const onFilesAdded = (files: File[]) => files.forEach(file => session.addFile(file));

  return (
    <div className="content main-page">
      <div className="files-list">
        {session.files.map(file => (
          <FileButton seed={session.seed} file={file} />
        ))}
        <FileInput onDrop={onFilesAdded} />
      </div>
      <div className="separator">or</div>
      <div className="peer-form">
        <input
          className="session-code-input input"
          type="text"
          value={peerMnemonics}
          onChange={event => setPeerMnemonics(event.currentTarget.value)}
        />
        <button
          className="button"
          onClick={() => {
            session.addPeer(peerMnemonics);
            setPeerMnemonics('');
          }}
        >
          {LOCALIZATION.addPeerButton}
        </button>
      </div>
    </div>
  );
};
