use git2::{FetchOptions, Repository};

fn main() -> Result<(), git2::Error> {
    // Open the local repository
    let repo = Repository::open(".")?;

    // Fetch the latest changes from the remote repository
    let mut fetch_options = FetchOptions::new();
    fetch_options.update_fetchhead(true);
    let mut remote = repo.find_remote("origin")?;

    let branch = repo.find_remote(repo.head()?.name().expect("utf8 branch name"))?;

    remote.fetch(&[branch.name().unwrap()], Some(&mut fetch_options), None)?;

    // Get the local and remote HEADs
    let local_head = repo.head()?.peel_to_commit()?;
    let fetch_head = repo.find_reference("FETCH_HEAD")?.peel_to_commit()?;

    // Compare the local and remote HEADs
    if dbg!(local_head.id()) != dbg!(fetch_head.id()) {
        println!("The repository is out of date");
    } else {
        println!("The repository is up to date");
    }

    Ok(())
}
