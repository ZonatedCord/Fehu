#!/bin/bash
echo "Rimozione blocco Gatekeeper da Fehu..."
xattr -dr com.apple.quarantine /Applications/Fehu.app 2>/dev/null
if [ $? -eq 0 ]; then
    echo "✓ Fatto! Puoi avviare Fehu normalmente."
else
    echo "Fehu non trovata in /Applications — trascinala prima nella cartella Applicazioni."
fi
echo ""
echo "Premi Invio per chiudere..."
read
