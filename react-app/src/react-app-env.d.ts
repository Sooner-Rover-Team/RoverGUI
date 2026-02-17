// Environment file to declare types for CSS modules and other assets
// Without this TypeScript doesn't recognize CSS and will yell at you
declare module "*.css" {
    const content: Record<string, string>;
    export default content;
}