import {
  configureWunderGraphApplication,
  cors,
  introspect,
} from '@wundergraph/sdk';
import { NextJsTemplate } from '@wundergraph/nextjs/dist/template';
import server from './generated/wundergraph.server';
import operations from './generated/wundergraph.operations';
import nodes from './generated/wundergraph.webhooks';

const postgresql = introspect.postgresql({
  apiNamespace: 'db',
  databaseURL: process.env.DATABASE_URL || 'postgres://aruaru:aruaru@127.0.0.1:5432/aruaru',
});

const aruaruPoem = introspect.openApiV2({
  apiNamespace: 'aruaru',
  source: {
    kind: 'file',
    filePath: './openapi/aruaru-poem.json',
  },
});

configureWunderGraphApplication({
  apis: [postgresql, aruaruPoem],
  server,
  operations,
  webhooks: nodes,
  generate: {
    codeGenerator: {
      templates: [new NextJsTemplate()],
    },
  },
  cors: {
    ...cors.allowAll,
    allowedOrigins: process.env.WG_ALLOWED_ORIGIN
      ? [process.env.WG_ALLOWED_ORIGIN]
      : ['http://localhost:3000', 'http://127.0.0.1:3000', 'http://localhost:5173'],
  },
  security: {
    enableGraphQLEndpoint: false,
  },
});
