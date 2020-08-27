import React from 'react';

import {
  SessionSocketBuilder,
  SessionSocket,
  WsRequest
} from './sessionSocket';

export type IStateContext =
  | { state: 'uninitialized' }
  | { state: 'created'; phrase: string; connect: (phrase: string) => void }
  | { state: 'connected'; seed: string };

export const StateContext = React.createContext<IStateContext>({
  state: 'uninitialized'
});

let wsSocket: SessionSocket | null = null;

export class State extends React.Component<{}, IStateContext> {
  private builder: SessionSocketBuilder;

  constructor(props: {}) {
    super(props);

    this.builder = new SessionSocketBuilder()
      .onCreate(phrase => {
        this.setState({
          state: 'created',
          phrase,
          connect: phrase => {
            wsSocket?.send('connect', { phrase });
          }
        });
      })
      .onConnect(seed => {
        this.setState({
          state: 'connected',
          seed
        });
      })
      .onPeerNotFound(() => {
        alert('Peer not found!');
      });

    this.state = {
      state: 'uninitialized'
    };
  }

  componentDidMount() {
    console.log(wsSocket);
    wsSocket?.close();
    wsSocket = this.builder.build();
    console.log(wsSocket);
  }

  render() {
    return (
      <StateContext.Provider value={this.state}>
        {this.props.children}
      </StateContext.Provider>
    );
  }
}
