function login() {
    var username = document.getElementById('username').value;
    var password = document.getElementById('password').value;

    var data = {
        user: username,
        password: password
    };

    fetch(`/ppm/login`, {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
        },
        body: JSON.stringify(data)
    })
        .then(
            response => response.json()
        )
        .then(
            data => {
                // console.log(data);
                localStorage.setItem('TOKEN', data.token);
                // Rediriger vers la page "search_page"
                window.location.href = '/ppm/search_page';
            })
        .catch(error => {
            console.error('Une erreur s\'est produite:', error);
        });

}

function navigateInputPage(uuid) {
    localStorage.setItem("SELECTED_ENTRY", uuid);
    window.location.href = '/ppm/input_page';
}


function navigateSetupPage() {
    window.location.href = '/ppm/setup_page';
}

async function showPass(encrypted) {
    const clear = await decrypt(encrypted);
    alert(clear);
}

async function logout(event) {

    // Récupérer la valeur du stockage local
    var global_token = localStorage.getItem('TOKEN');

    fetch(`/ppm/logout`, {
        headers: {
            'Content-Type': 'application/json',
            'token_id': global_token
        }
    })
        .then(response => response.json())
        .then(data => {
            // Traitement des résultats JSON
            console.log(data);
            window.location.href = '/ppm/index'
        })
        .catch(error => {
            console.error('Une erreur s\'est produite:', error);
        });
}

function performSearch(event) {
    event.preventDefault();
    const searchInput = document.getElementById('chars');
    const searchText = searchInput.value;

    // Récupérer la valeur du stockage local
    var global_token = localStorage.getItem('TOKEN');

    fetch(`/ppm/search?chars=${encodeURIComponent(searchText)}`, {
        headers: {
            'Content-Type': 'application/json',
            'token_id': global_token
        }
    })
        .then(response => response.json())
        .then(data => {
            // Traitement des résultats JSON
            console.log(data);
            drawResult(data);
        })
        .catch(error => {
            console.error('Une erreur s\'est produite:', error);
        });
}


function createField(label, value, style) {

    const inputItem = document.createElement('div');
    inputItem.classList.add('result-value');
    if (style !== null || style !== 'undefined' ) {
       inputItem.classList.add(style);
    }

    const labelItem = document.createElement('label');
    labelItem.textContent = label + ': ';
    inputItem.appendChild(labelItem);

    const inputField = document.createElement('input');
    inputField.setAttribute('readonly', true);
    inputField.value = value;
    inputItem.appendChild(inputField);

    return inputItem;
}

function drawResult(results) {
    const resultsContainer = document.getElementById('results-container');
    resultsContainer.classList.add('results-grid');
    resultsContainer.innerHTML = '';
    results.forEach(result => {
        var block = drawEntry(result);
        resultsContainer.appendChild(block);
    });
}

function drawEntry(result) {

    const block = document.createElement('div');
    block.classList.add('result-block');
    localStorage.setItem(result.uuid, JSON.stringify(result));
    block.appendChild(createField('UUID', result.uuid));
    block.appendChild(createField('Entry ID', result.entry_id));
    block.appendChild(createField('Titre', result.title, 'result-value-title'));
    //block.appendChild(createField('Order', result.order));
    block.appendChild(createField("Nom d'utilisateur", result.username));
    block.appendChild(createField('Mot de passe', "•••••••" ));
    // block.appendChild(createField('Mot de passe', result.encrypted_pass));
    block.appendChild(createField('Site Web', result.url));
    block.appendChild(createField('Notes', result.notes));
    const catEl = createField('Catégorie', result.category)
    if (result.category !== null && result.category.trim() !== '' ) {
        catEl.style.backgroundColor = generateRGBFromString(result.category);
    }
    block.appendChild(catEl);
    // block

    // Créer un élément bouton
    var button = document.createElement('button');
    button.textContent = 'Modifier';
    // Ajouter l'événement onclick
    const uuid = result.uuid;
    button.onclick = () => navigateInputPage(uuid);
    block.appendChild(button);

    // Créer un élément bouton "voir"
    var bttnShow = document.createElement('button');
    bttnShow.textContent = 'Voir Mot de passe';
    bttnShow.onclick = () => showPass(result.encrypted_pass);
    block.appendChild(bttnShow);

    return block;
}

  function handleKeyPress(event) {
    if (event.keyCode === 13) {
      performSearch(event);
    }
  }

async function loadCategory() {
    const global_token = localStorage.getItem('TOKEN');
    try {
        const response = await fetch('/ppm/categories', {
            headers: {
                'Content-Type': 'application/json',
                'token_id': global_token
            }
        });
        // if (!response.ok) {
        //     throw new Error('Erreur lors de la requête');
        // }

        const data = await response.json();
        const categoriesJson = JSON.stringify(data)
        localStorage.setItem('PPM_CATEGORY', categoriesJson)
    } catch (error) {
        console.error('Une erreur s\'est produite pendant la lecture des catégories:', error);
    }
}

async function decrypt(encrypted) {
    const global_token = localStorage.getItem('TOKEN');
    try {
        const response = await fetch(`/ppm/decrypt/${encodeURIComponent(encrypted)}`, {
            headers: {
                'Content-Type': 'application/json',
                'token_id': global_token
            }
        });
        // if (!response.ok) {
        //     throw new Error('Erreur lors de la requête');
        // }

        const data = await response.json();
        const clearText = data.clear_text;
        return clearText;
    } catch (error) {
        console.error('Une erreur s\'est produite:', error);
    }
}

async function fillEntry() {
    console.log("start fill entry");
    const uuid = localStorage.getItem('SELECTED_ENTRY');
    if (uuid !== null && uuid !== "") {
        var data = JSON.parse(localStorage.getItem(uuid));
        var clear = await decrypt(data.encrypted_pass);
        console.log(clear);
        document.getElementById('uuid').value = data.uuid;
        document.getElementById('entry_id').value = data.entry_id;
        document.getElementById('title').value = data.title;
        document.getElementById('username').value = data.username;
        document.getElementById('password').value = clear;
        document.getElementById('url').value = data.url;
        document.getElementById('notes').value = data.notes;
        document.getElementById('category').value = data.category;
    }
    console.log(data);
}

function buildCatSelector() {

    const listOfCategories = JSON.parse(localStorage.getItem('PPM_CATEGORY'))

    // Remplir la liste des options avec les valeurs du tableau listOfCategories
    $.each(listOfCategories, function(index, value) {
        $('#categoryOptions').append('<option>' + value + '</option>');
    });

    // Afficher la boîte de sélection lorsque vous cliquez sur l'emoji œil
    $('#eye').click(function() {
        $('#categorySelection').show();
    });

    // Renvoyer la valeur sélectionnée dans l'input "category" lorsque vous choisissez une option
    $('#categoryOptions').change(function() {
        var selectedCategory = $(this).val();
        $('#category').val(selectedCategory);
        $('#categorySelection').hide();
    });

    // Masquer la boîte de sélection lorsque vous cliquez en dehors de celle-ci
    $(document).mouseup(function(e) {
        var container = $("#categorySelection");
        if (!container.is(e.target) && container.has(e.target).length === 0) {
            container.hide();
        }
    });

}

async function saveUser() {

    var username = document.getElementById('username').value;
    var password = document.getElementById('password').value;
    // Récupérer la valeur du stockage local
    var global_token = localStorage.getItem('TOKEN');

    var form = {
        "user": username,
        "password": password,
    };

    $('#confirmation-box').fadeIn();

    await postData(global_token, form, '/ppm/setup', null)
}

async function saveEntry() {
    var entry_id = document.getElementById('entry_id').value;
    var title = document.getElementById('title').value;
    var username = document.getElementById('username').value;
    var password = document.getElementById('password').value;
    var url = document.getElementById('url').value;
    var notes = document.getElementById('notes').value;
    var category = document.getElementById('category').value;

    // Récupérer la valeur du stockage local
    var global_token = localStorage.getItem('TOKEN');

    if (entry_id.trim() === '') {
        entry_id = null;
    }

    var form = {
        "entry_id": entry_id,
        "title": title,
        "username": username,
        "pass": password,
        "url": url,
        "notes": notes,
        "category": category
    };

    $('#confirmation-box').fadeIn();

    await postData(global_token, form, '/ppm/add_entry', loadCategory )
}

async function postData(global_token, form, url, success_callback) {
    try {
        const response = await fetch(url, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'token_id': global_token
            },
            body: JSON.stringify(form)
        });

        const data = await response.json();
        console.log(data);

        setTimeout(function() {
            $('#confirmation-box').fadeOut();
        }, 700);

        // Recharge la liste des catégories
        if (success_callback !== null) {
            await success_callback(); // loadCategory();
        }
    } catch (error) {
        console.error('Une erreur s\'est produite:', error);

        setTimeout(function() {
            $('#confirmation-box').fadeOut();
        }, 700);
    }
}

function generateRGBFromString(str) {
    const partLength = Math.ceil(str.length / 3);

    const part1 = str.slice(0, partLength);
    const part2 = str.slice(partLength, 2 * partLength);
    const part3 = str.slice(2 * partLength);

    const r = fromStrToColor(part1);
    const g = fromStrToColor(part2);
    const b = fromStrToColor(part3);

    // Retourner le code couleur RGB sous forme d'une chaîne
    return `rgb(${r}, ${g}, ${b})`;
}

function fromStrToColor(part) {
    // Calculer la somme des codes ASCII des caractères dans la chaîne
    let sum = 0;
    for (let i = 0; i < part.length; i++) {
        sum += part.charCodeAt(i);
    }
    const c = sum % 256;
    return c;
}

