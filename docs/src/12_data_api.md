<!-- Loading swagger only for this page (2Mb of js damn)... -->
<link rel="stylesheet" href="./static/swagger-ui.css">
<style>
    :root {
        --table-alternate-bg: transparent;
        --table-border-color: transparent;
    }
</style>
<script src="./static/swagger-ui-bundle.js"></script>
<script src="./static/swagger.component.js"></script>

<div>
    <nut-swagger spec-url="./static/openapi3_spec.json"></nut-swagger>
</div>
