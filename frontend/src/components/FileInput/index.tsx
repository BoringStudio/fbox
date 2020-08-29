import React from 'react';

import { useDropzone } from 'react-dropzone';

const LOCALIZATION = {
  label: (isActive: boolean) =>
    isActive ? 'Drop the files here...' : "Drag'n'drop some files here, or click to select files"
};

export type IProps = {
  onDrop: (files: File[]) => void;
};

export const FileInput = (props: IProps) => {
  const onDrop = React.useCallback(
    acceptedFiles => {
      props.onDrop(acceptedFiles);
    },
    [props]
  );

  const { getRootProps, getInputProps, isDragActive } = useDropzone({ onDrop });

  return (
    <div {...getRootProps({})}>
      <input {...getInputProps()} />
      <p>{LOCALIZATION.label(isDragActive)}</p>
    </div>
  );
};
