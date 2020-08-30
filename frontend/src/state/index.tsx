import React from 'react';

import { v4 as uuidv4 } from 'uuid';
import { SessionSocketBuilder, SessionSocket, FileInfo } from './sessionSocket';

export const StateContext = React.createContext<PossibleStates>({
  kind: 'uninitialized'
});

let wsSocket: SessionSocket | null = null;

export class State extends React.Component<{}, PossibleStates> {
  private builder: SessionSocketBuilder;

  constructor(props: {}) {
    super(props);

    const addPeer = (phrase: string) => wsSocket?.send('connect', { phrase });

    const addFile = (file: File) => {
      if (this.state.kind !== 'connected') {
        return;
      }

      const { localFiles, ...rest } = this.state;
      const id = uuidv4();

      this.setState(
        {
          ...rest,
          localFiles: localFiles.set(id, file)
        },
        () => {
          wsSocket?.send('add_file', {
            id,
            name: file.name,
            mime_type: file.type,
            size: file.size
          });
        }
      );
    };

    const removeFile = (id: string) => {
      if (this.state.kind !== 'connected') {
        return;
      }

      const { localFiles, ...rest } = this.state;
      localFiles.delete(id);

      this.setState(
        {
          ...rest,
          localFiles
        },
        () => wsSocket?.send('remove_file', { id })
      );
    };

    this.builder = new SessionSocketBuilder();
    this.builder.responseHandlers = {
      created: ({ phrase }) => {
        this.setState({
          kind: 'created',
          phrase,
          addPeer
        });
      },
      connected: ({ connection_id, seed, files }) => {
        this.setState({
          kind: 'connected',
          connection_id,
          seed,
          files,
          localFiles: new Map(),
          addPeer,
          addFile,
          removeFile,
          reconnect: this.reconnect
        });
      },
      file_added: file => {
        if (this.state.kind !== 'connected') {
          return;
        }

        const { files, ...rest } = this.state;
        this.setState({
          ...rest,
          files: [...files, file]
        });
      },
      file_removed: ({ id }) => {
        if (this.state.kind !== 'connected') {
          return;
        }

        const { files, ...rest } = this.state;
        this.setState({
          ...rest,
          files: files.filter(file => file.id !== id)
        });
      },
      file_requested: ({ id }) => {
        if (this.state.kind !== 'connected') {
          return;
        }

        const localFile = this.state.localFiles.get(id);
        if (localFile == null) {
          return;
        }

        const url = `${process.env.REACT_APP_API_URL}/sessions/files/${id}`;

        fetch(url, {
          method: 'POST',
          body: localFile,
          headers: {
            'X-Session-Seed': this.state.seed,
            'Content-Type': 'application/octet-stream'
          }
        })
          .then(console.log)
          .catch(console.warn);
      },
      peer_not_found: () => {
        alert('Peer not found!');
      },
      session_not_found: () => {
        alert('Session not found!');
      },
      file_count_limit_reached: () => {
        alert('File count limit reached!');
      }
    };

    this.state = {
      kind: 'uninitialized'
    };
  }

  reconnect = () => {
    wsSocket?.close();
    wsSocket = this.builder.build();
    console.log(wsSocket);
  };

  componentDidMount() {
    this.reconnect();
  }

  render() {
    return <StateContext.Provider value={this.state}>{this.props.children}</StateContext.Provider>;
  }
}

// state content types

type IStateContent = {
  uninitialized: {};
  created: {
    phrase: string;
    addPeer: (phrase: string) => void;
  };
  connected: {
    connection_id: number;
    seed: string;
    files: FileInfo[];
    localFiles: Map<string, File>;
    addPeer: (phrase: string) => void;
    addFile: (file: File) => void;
    removeFile: (id: string) => void;
    downloadFile: (id: string) => void;
    reconnect: () => void;
  };
};

type IStateContentType = keyof IStateContent;

export type IStateContext<T extends IStateContentType> = { kind: T } & IStateContent[T];

type PossibleStates = IStateContext<'uninitialized'> | IStateContext<'created'> | IStateContext<'connected'>;
