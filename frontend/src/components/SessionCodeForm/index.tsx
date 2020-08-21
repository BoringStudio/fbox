import React from 'react';

import './style.scss';

export const SessionCodeForm = () => {
  return (
    <div className="session-code-form">
      <div className="title noselect">FBox</div>
      <hr />
      <div className="description noselect">
        Scan QR code on the left with your second device or enter mnemonics
        below
      </div>
      <input className="session-code-input" type="text"></input>
      <button className="join-button">Join</button>
    </div>
  );
};
