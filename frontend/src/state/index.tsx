import React from 'react';

import { SessionSocketBuilder, SessionSocket } from './sessionSocket';

export type IStateUninitialized = {
  kind: 'uninitialized';
};

export type IStateCreated = {
  kind: 'created';
  phrase: string;
  connect: (phrase: string) => void;
};

export type IStateConnected = {
  kind: 'connected';
  seed: string;
  addPeer: (phrase: string) => void;
  reconnect: () => void;
};

export type IStateContext = IStateUninitialized | IStateCreated | IStateConnected;

export const StateContext = React.createContext<IStateContext>({
  kind: 'uninitialized'
});

let wsSocket: SessionSocket | null = null;

export class State extends React.Component<{}, IStateContext> {
  private builder: SessionSocketBuilder;

  constructor(props: {}) {
    super(props);

    const addPeer = (phrase: string) => wsSocket?.send('connect', { phrase });

    this.builder = new SessionSocketBuilder();
    this.builder.responseHandlers = {
      created: ({ phrase }) => {
        this.setState({
          kind: 'created',
          phrase,
          connect: addPeer
        });
      },
      connected: ({ seed }) => {
        this.setState({
          kind: 'connected',
          seed,
          addPeer: addPeer,
          reconnect: this.reconnect
        });
      },
      peer_not_found: () => {
        alert('Peer not found!');
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
