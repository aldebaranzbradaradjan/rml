use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

pub fn preprocess_rml(input: &TokenStream) -> String {
    

    // ici on veut préparser l'input (un fichier rml)
    // et retourner le code rml étendu avec les composants importés
    
    // On va donc proceder ainsi; d'abors identifier les imports
    // quand on trouve un "import" par exemple :
    // import "test_components" as UI
    // on va ajouter le path "test_components" à une hsahmap avec la clef "UI"
    // cette hashmap servira juste après pour ajouter les composant importés par le fichier rml
    // par exemple : 
    // UI::SimpleCard { id: my_card }
    // on va remplacer UI::SimpleCard par le contenu de test_components/SimpleCard.rml
    // en prenant soin de :
    // - remplacer les id des composants SimpleCard dans par des id uniques pour chaque composant du fichier (et éventuellement les remplacer dans les callbacks et méthodes s'il y en a dans le fichier SimpleCard.rml)
    // - remplacer l'id racine de SimpleCard.rml si il est spécifié (dans l'exemple ci-dessus, my_card) sinon le remplacer par un id unique (et éventuellement les remplacer dans les callbacks et méthodes s'il y en a dans le fichier SimpleCard.rml)
    // - écraser les propriétés (et ou signal, callback, methode) du composant racine de SimpleCard.rml si elles sont spécifiées dans l'instanciation
    //      par exemple : UI::SimpleCard { id: my_card, color: { RED } }
    // - ajouter les nouvelle propriétés (et ou signal, callback, methode) définies dans l'instance si elle ne sont pas présente dans le composant racine de SimpleCard.rml
    //      par exemple : UI::SimpleCard { id: my_card, new_color: { RED } }

    // et à chaque import de fichier rml, recommencer l'opération, en rappelant preprocess_rml dessus

    input.to_string()
}