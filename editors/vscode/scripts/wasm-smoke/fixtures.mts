export const workspaceDocuments = [
  {
    uri: "file:///workspace/acme.scm",
    text: '(library (acme tools) (export acme-tool acme-helper) (doc-next (signature "(acme-tool name [mode])") (summary "Acme.") (param \'name "Name docs.")) (define (acme-tool name) name) (define (acme-helper value) value))',
  },
];

export const workspace = JSON.stringify(workspaceDocuments);

export const documentText =
  '(import (acme tools))\n(acme-tool "demo")\n(acme-helper "demo")\n(define (local-helper value) value)\n(local-helper "demo")';
