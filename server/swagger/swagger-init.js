window.addEventListener('DOMContentLoaded', function () {
  window.ui = SwaggerUIBundle({
    url: '/api/openapi.yaml',
    dom_id: '#swagger-ui',
    presets: [SwaggerUIBundle.presets.apis],
    layout: 'BaseLayout',
    deepLinking: true,
    displayRequestDuration: true,
    docExpansion: 'list',
    defaultModelsExpandDepth: 0,
    persistAuthorization: true,
  })
})
