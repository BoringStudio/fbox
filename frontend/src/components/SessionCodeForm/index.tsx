import React from 'react';

import './style.scss';

const LOCALIZATION = {
  title: 'FBox',
  description:
    'Scan QR code on the left with your second device or enter mnemonics below:',
  joinButton: 'Add Peer'
};

type SessionCodeFormProps = {
  onSubmit: (input: string) => void;
};

export const SessionCodeForm = (props: SessionCodeFormProps) => {
  const { onSubmit } = props;

  const [value, setValue] = React.useState<string>('');

  const onJoinButtonClick = () => {
    setValue('');
    onSubmit(value);
  };

  return (
    <div className="session-code-form">
      <div className="title noselect">{LOCALIZATION.title}</div>
      <hr />
      <div className="description noselect">{LOCALIZATION.description}</div>
      <input
        className="session-code-input input"
        type="text"
        value={value}
        onChange={event => setValue(event.currentTarget.value)}
      />
      <button className="button" onClick={onJoinButtonClick}>
        {LOCALIZATION.joinButton}
      </button>
    </div>
  );
};
