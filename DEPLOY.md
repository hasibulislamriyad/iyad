# Notch: Vercel e deploy korar niyom

## Part 1: Website live kora (10 minute, kono install lage na)
1. github.com-e account kholo, **New repository** e click koro, naam dao `notch`, **Public** rakho.
2. Repository page-e **uploading an existing file** e click koro. Ei folder-er shob kichu (site, desktop-app, .github, vercel.json) drag kore dao, **Commit changes** chap.
   (Windows Explorer-e `.github` folder hidden thakle View → Hidden items tick koro.)
3. vercel.com-e GitHub diye login koro → **Add New → Project** → `notch` repo-ta **Import** → **Deploy**. Kono setting bodlano lage na.
4. Kichu khon por tomar link pabe: `notch-xxxx.vercel.app`. Keu ekhon website khule demo chalate ar web version install korte parbe.

## Part 2: Windows .exe banano (kono PC install lage na, GitHub nijei banay)
1. GitHub repo → **Actions** tab → **Build Windows installer** → **Run workflow**.
2. 10-15 minute por repo-r **Releases** e `Notch_0.1.0_x64-setup.exe` thakbe.
3. Repo-te `site/index.html` kholo, pencil icon diye edit koro. `DOWNLOAD_URL` line-e `YOUR-NAME/YOUR-REPO` er jaygay tomar GitHub naam ar repo naam boshao. **Commit** koro.
4. Vercel nijei notun version publish kore dibe. Ekhon "Download Notch for Windows" button kaj korbe.

## Jodi Actions fail kore
Actions → fail howa run → logs copy kore Claude-ke pathao.
