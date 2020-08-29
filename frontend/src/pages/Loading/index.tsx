import React from 'react';

import './style.scss';

export const LoadingPage = () => {
  const [selectedFile, setSelectedFile] = React.useState<File | null>(null);

  console.log(selectedFile);

  const onFileSelected = (event: React.ChangeEvent<HTMLInputElement>) => {
    setSelectedFile(event.currentTarget.files?.[0] || null);
  };

  const onSubmit = (event: React.FormEvent<HTMLButtonElement>) => {
    event.preventDefault();

    if (selectedFile == null) {
      return;
    }

    fetch(new URL('v1/sessions/files', process.env.REACT_APP_API_URL).toString(), {
      method: 'POST',
      body: selectedFile,
      headers: {
        'Content-Type': 'application/octet-stream'
      }
    })
      .then(console.log)
      .catch(console.warn);
  };

  return (
    <div className="content loading-page">
      <div>
        <input type="file" name="file" onChange={onFileSelected} required />
        <button type="submit" className="button" disabled={selectedFile == null} onClick={onSubmit}>
          Submit
        </button>
      </div>
      <img src="/spinner.svg" alt="Loading..." />
    </div>
  );
};
