# Topaz Blog

This is the Hugo-powered blog for the Topaz programming language project. The blog is configured for deployment to GitHub Pages.

## Local Development

To run the blog locally:

```bash
cd docs
hugo server --buildDrafts
```

Visit http://localhost:1313/topaz/ to preview the site.

## Creating New Posts

```bash
cd docs
hugo new content posts/your-post-name.md
```

Edit the created file in `content/posts/` and set `draft = false` when ready to publish.

## Color Theme

The blog uses a clean white background with ocean blue accents (#0077BE). Custom styles are in [`assets/css/extended/custom.css`](assets/css/extended/custom.css).

## Deployment

The blog automatically deploys to GitHub Pages when you push to the main branch. The GitHub Actions workflow is defined in [`.github/workflows/hugo.yml`](../.github/workflows/hugo.yml).

### Setup GitHub Pages

1. Go to your repository's Settings → Pages
2. Set Source to "GitHub Actions"
3. The workflow will automatically build and deploy on push

## Theme

Uses [Hugo PaperMod](https://github.com/adityatelange/hugo-PaperMod) theme, configured as a git submodule.
